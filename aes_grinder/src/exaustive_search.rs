use crate::{algo::Algo, matrix::Matrix};
use std::{cmp::Ordering, collections::HashSet};

fn exhaustive_search<E>(mut x: Matrix, time_complexity: u32) -> HashSet<Box<Algo>> {
    //Set of base solvers
    let mut g:HashSet<Box<Algo>> = HashSet::new();
    for xVar in x.get_all_variables() {
        //We create a base solver for each variables for variables that are not S(x)
        if xVar.contains('(') {
            continue;
        }
        g.insert(Box::new(Algo::base_solver(&mut x, xVar)));
    }
    //Set of pair of algo
    let mut p:HashSet<(Box<Algo>, Box<Algo>)> = HashSet::new();
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
        let c = Box::new(Algo::fusion_two_algo(a1.clone(), a2.clone()));

        if c.get_time_complexity() <= time_complexity {
            update_queue(&mut g, &mut p, c);
        }
    };
    g
}

fn update_queue(g:&mut HashSet<Box<Algo>>, p:&mut HashSet<(Box<Algo>, Box<Algo>)>, c:Box<Algo>) -> () {
    //For all algo worst than c
    for a in g.iter() {
        match c.compare1(a) {
            Some(Ordering::Greater) => {
                let mut gdiff: HashSet<Box<Algo>> = g.iter().fold(HashSet::<Box<Algo>>::new(), |mut s, elt| -> HashSet<Box<Algo>> {
                    if c.compare1(elt) == Some(Ordering::Greater) {
                        s.insert(elt.clone());
                    }
                    s
                });
                g.insert(c.clone());
                let gprim = g.difference(&mut gdiff).cloned().collect::<HashSet<Box<Algo>>>();
                let mut pdiff = p.iter().fold(HashSet::<(Box<Algo>, Box<Algo>)>::new(), |mut s, elt| -> HashSet<(Box<Algo>, Box<Algo>)> {
                    if c.compare1(&elt.0) == Some(Ordering::Greater) || c.compare1(&elt.1) == Some(Ordering::Greater) {
                        s.insert(elt.clone());
                    }
                    s
                });
                let pprim = p.difference(&mut pdiff).cloned().collect::<HashSet<(Box<Algo>, Box<Algo>)>>();
                let uprim =  pprim.union(&mut gprim.iter().fold(HashSet::<(Box<Algo>, Box<Algo>)>::new(), |mut s, elt| -> HashSet<(Box<Algo>, Box<Algo>)> {
                    let vc = c.get_all_variables();
                    let velt = elt.get_all_variables();
                    if !vc.is_subset(&velt) && !velt.is_subset(&vc) {
                        s.insert((c.clone(), elt.clone()));
                    }
                    s
                })).cloned().collect::<HashSet<(Box<Algo>, Box<Algo>)>>();
                //Find a means to update g and p
                *g = gprim;
                *p = uprim;
                todo!();
            },
            _ => continue,
        }
    }
} 