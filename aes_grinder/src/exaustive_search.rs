use crate::{algo::Algo, matrix::Matrix};
use std::{cmp::Ordering, collections::HashSet};

pub fn exhaustive_search(mut x: Matrix, time_complexity: usize) -> HashSet<Box<Algo>> {
    //Set of base solvers
    let mut g: HashSet<Box<Algo>> = HashSet::new();
    for x_var in x.get_all_variables() {
        //We create a base solver for each variables for variables that are not S(x)
        if x_var.contains('(') {
            continue;
        }
        g.insert(Box::new(Algo::base_solver(&mut x, x_var)));
    }
    //Set of pair of algo
    let mut p: HashSet<(Box<Algo>, Box<Algo>)> = HashSet::new();
    g.clone().iter().enumerate().for_each(|(i, a1)| {
        g.clone().iter().enumerate().for_each(|(j, a2)| {
            if i < j {
                p.insert((a1.clone(), a2.clone()));
            }
        });
    });

    while !p.is_empty() {
        //Take a pair of algo from p
        let (a1, a2) = p.iter().next().unwrap();
        let c = Box::new(Algo::fusion_two_algo(a1.clone(), a2.clone(), &mut x));

        if c.get_time_complexity() <= time_complexity {
            update_queue(&mut g, &mut p, c);
        }
    }
    g
}

// function UPDATE-QUEUE(G, P, A):
//     if there is no A' in G such that A' dominates A:
//         G' = {A} union G minus {A' in G: A dominates A'}
//         P' = P minus {(A1, A2) in P: A dominates A1 or A dominates A2}
//         P' = P' union {(A, A'): A' in G', vertices of A not in vertices of A' and vertices of A' not in vertices of A}
//     end if
//     return (G', P')
// end function
fn update_queue(
    g: &mut HashSet<Box<Algo>>,
    p: &mut HashSet<(Box<Algo>, Box<Algo>)>,
    c: Box<Algo>,
) -> () {
    let mut dominated = false;
    g.iter().for_each(|a| {
        if let Some(Ordering::Greater) = c.compare1(a) {
            dominated = true;
        }
    });

    if !dominated {
        // Calculate G' as {A} union G minus {A' in G: A dominates A'}
        let gdiff: HashSet<Box<Algo>> = g.iter().fold(
            HashSet::<Box<Algo>>::new(),
            |mut s, elt| -> HashSet<Box<Algo>> {
                if c.compare1(elt) == Some(Ordering::Greater) {
                    s.insert(elt.clone());
                }
                s
            },
        );

        // Calculate G' as {A} union G minus {A' in G: A dominates A'}
        let gprim: HashSet<Box<Algo>> = g.difference(&gdiff).cloned().collect();

        // Calculate P' as P minus {(A1, A2) in P: A dominates A1 or A dominates A2}
        let pdiff = p.iter().fold(
            HashSet::<(Box<Algo>, Box<Algo>)>::new(),
            |mut s, elt| -> HashSet<(Box<Algo>, Box<Algo>)> {
                if c.compare1(&elt.0) == Some(Ordering::Greater)
                    || c.compare1(&elt.1) == Some(Ordering::Greater)
                {
                    s.insert(elt.clone());
                }
                s
            },
        );
        let pprim = p
            .difference(&pdiff)
            .cloned()
            .collect::<HashSet<(Box<Algo>, Box<Algo>)>>();

        // Calculate P' as P' union {(A, A'): A' in G', vertices of A not in vertices of A' and vertices of A' not in vertices of A}
        let uprim: HashSet<(Box<Algo>, Box<Algo>)> = pprim
            .union(&gprim.iter().fold(
                HashSet::<(Box<Algo>, Box<Algo>)>::new(),
                |mut s, elt| -> HashSet<(Box<Algo>, Box<Algo>)> {
                    let vc = c.get_all_variables();
                    let velt = elt.get_all_variables();
                    if !vc.is_subset(&velt) && !velt.is_subset(&vc) {
                        s.insert((c.clone(), elt.clone()));
                    }
                    s
                },
            ))
            .cloned()
            .collect();

        // Update g and p
        *g = gprim;
        *p = uprim;
    }
}
