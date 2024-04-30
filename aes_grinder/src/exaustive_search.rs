use crate::{algo::Algo, matrix::Matrix};
use rand::{seq::IteratorRandom, Rng};
use std::{cmp::Ordering, collections::HashSet};

pub fn random_search(mut matrix: Matrix) -> Box<Algo> {
    //Set of base solvers
    let mut lst_algo: Vec<Box<Algo>> = vec![];

    for x_var in matrix.get_all_variables() {
        //We create a base solver for each variables for variables that are not S(x)
        if x_var.contains('(') {
            continue;
        }
        lst_algo.push(Box::new(Algo::base_solver(&mut matrix, x_var)));
    }

    let mut size_of_lst = lst_algo.len();
    while size_of_lst > 1 {
        let first_rand = rand::thread_rng().gen_range(0..size_of_lst);
        size_of_lst -= 1;
        let second_rand = rand::thread_rng().gen_range(0..size_of_lst);
        let first_algo = lst_algo.remove(first_rand);
        let second_algo = lst_algo.remove(second_rand);
        lst_algo.push(Box::new(Algo::fusion_two_algo(
            first_algo,
            second_algo,
            &mut matrix,
        )));
    }

    lst_algo.pop().unwrap()
}

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
    println!("G init");
    print_algo_set(&g);
    //Set of pair of algo
    let mut p: HashSet<(Box<Algo>, Box<Algo>)> = HashSet::new();
    g.clone().iter().enumerate().for_each(|(i, a1)| {
        g.clone().iter().enumerate().for_each(|(j, a2)| {
            if i < j {
                // println!("a1 : {:?}", a1.clone());
                // println!("a2 : {:?}", a2.clone());
                p.insert((a1.clone(), a2.clone()));
            }
        });
    });
    println!("P init");
    print_pair_algo_set(&p);
    let mut rng = rand::thread_rng();
    // While g dont contains an algo with 20 variables
    while !p.is_empty() {
        //Take a pair of algo from p
        //println!(" plen : {:?}", p.len());
        let a = p.iter().choose(&mut rng).unwrap().clone();
        let (a1, a2) = p.take(&a).unwrap();

        //println!("take : \n{:?} and {:?}", a1.get_all_variables(),a2.get_all_variables());
        let c = Box::new(Algo::fusion_two_algo(a1.clone(), a2.clone(), &mut x));
        //println!("New algo : \n{:?}", c);
        if c.get_time_complexity() <= time_complexity {
            update_queue(&mut g, &mut p, c);
        } else {
            println!("Time complexity reached")
        }
        //print_algo_set(&g);
        //print_pair_algo_set(&p);
    }
    g
}

fn print_algo_set(g: &HashSet<Box<Algo>>) {
    println!("g ({}):", g.len());
    g.iter().for_each(|a| {
        println!("algo : {:?}", a.get_all_variables());
    });
}

fn print_pair_algo_set(p: &HashSet<(Box<Algo>, Box<Algo>)>) {
    println!("p ({}):", p.len());
    p.iter().for_each(|(a1, a2)| {
        println!(
            "Pair : ({:?}, {:?})",
            a1.get_all_variables(),
            a2.get_all_variables()
        );
    });
}

// function UPDATE-QUEUE(G, P, A):
//     if there is no A' in G such that A' dominates A:
//         G' = {A} union G minus {A' in G: A dominates A'}
//         P' = P minus {(A1, A2) in P: A dominates A1 or A dominates A2}
//         P' = P' union {(A, A'): A' in G', vertices of A not in vertices of A' and vertices of A' not in vertices of A}
//     end if
//     return (G', P')
// end function
fn update_queue(g: &mut HashSet<Box<Algo>>, p: &mut HashSet<(Box<Algo>, Box<Algo>)>, c: Box<Algo>) {
    //Check if there exists an Algo worth than c
    for aprim in g.iter() {
        if !(aprim >= &c) {
            continue;
        } else {
            return;
        }
    }

    // Keep all algo that are better or equal to c
    g.retain(|elt| {
        if Some(Ordering::Less) == elt.compare1(&c) {
            false
        } else {
            true
        }
    });

    // Add the new algo to the set
    g.insert(c.clone());

    // Keep all pair that are better or equal to c
    p.retain(|(a1, a2)| {
        if a1.compare1(&c) == Some(Ordering::Less) || a2.compare1(&c) == Some(Ordering::Less)
        {
            false
        } else {
            true
        }
    });

    //Form new pairs with the algos such as the variables of one are not a subset of the other 
    let pairs = g.iter().flat_map(|a| {
        if c.get_all_variables().is_subset(&a.get_all_variables()) ||
            a.get_all_variables().is_subset(&c.get_all_variables()) {
            None
        } else {
            Some((c.clone(), a.clone()))
        }
    });
    //Insert the new pairs in p
    p.extend(pairs);
}

#[cfg(test)]
mod test_exhaustive {

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_exhaustive1() {
        let mut matrix = Matrix::from(vec![vec![1, 2, 3], vec![4, 3, 2], vec![4, 8, 2]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        matrix.set_vars_map(vars_maps.clone());
        exhaustive_search(matrix, 50);
    }
    #[test]
    fn test_update_queue() {
        let mut matrix = Matrix::from(vec![
            vec![1, 4, 1, 1],
            vec![0, 1, 1, 0],
            vec![0, 0, 0, 1],
            vec![0, 7, 0, 1],
        ]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        vars_maps.insert("D".to_string(), 3);
        matrix.set_vars_map(vars_maps.clone());
        let c1 = Algo::base_solver(&mut matrix, "A".to_string());
        let c2 = Algo::base_solver(&mut matrix, "B".to_string());
        let c3 = Algo::base_solver(&mut matrix, "C".to_string());
        let c4 = Algo::base_solver(&mut matrix, "D".to_string());
        let c = Algo::fusion_two_algo(Box::new(c1.clone()), Box::new(c2.clone()), &mut matrix);

        let mut g: HashSet<Box<Algo>> = HashSet::new();
        let mut p: HashSet<(Box<Algo>, Box<Algo>)> = HashSet::new();
        g.insert(Box::new(c1.clone()));
        g.insert(Box::new(c2.clone()));
        g.insert(Box::new(c3.clone()));
        g.insert(Box::new(c4.clone()));

        p.insert((Box::new(c1.clone()), Box::new(c2.clone())));
        p.insert((Box::new(c1.clone()), Box::new(c3.clone())));
        p.insert((Box::new(c1.clone()), Box::new(c4.clone())));
        p.insert((Box::new(c2.clone()), Box::new(c3.clone())));
        p.insert((Box::new(c2.clone()), Box::new(c4.clone())));
        p.insert((Box::new(c3.clone()), Box::new(c4.clone())));
        update_queue(&mut g, &mut p, Box::new(c.clone()));
        println!("p : {:?}", p);
        println!("g : {:?}", g);
    }
}
