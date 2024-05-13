use crate::{algo::Algo, matrix::Matrix};
use rand::{seq::IteratorRandom, Rng};
use std::{cmp::Ordering, collections::HashSet, fmt::Display};
use strum::{EnumCount, EnumIter};

#[derive(EnumIter, EnumCount)]
pub enum Search {
    Exhaustive,
    Random,
}

impl Display for Search {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Search::Exhaustive => f.write_str("Exhaustive search"),
            Search::Random => f.write_str("Random search"),
        }
    }
}

pub fn random_search(matrix: &mut Matrix) -> Box<Algo> {
    //Set of base solvers
    let mut lst_algo: Vec<Box<Algo>> = vec![];

    for x_var in matrix.get_all_variables() {
        //We create a base solver for each variables for variables that are not S(x)
        if x_var.contains('(') {
            continue;
        }
        lst_algo.push(Box::new(Algo::base_solver(x_var)));
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
            matrix,
        )));
    }

    lst_algo.pop().unwrap()
}

pub fn search_best_multiple_random(matrix: &mut Matrix, nb_algo: usize) -> Box<Algo> {
    let mut g: HashSet<Box<Algo>> = HashSet::new();
    for _ in 0..nb_algo {
        g.insert(random_search(matrix));
    }
    //Find the best algo
    let best_algo = g.iter().max().unwrap().clone();
    best_algo
}

fn generate_all_base_solver(x: &Matrix) -> HashSet<Box<Algo>> {
    let mut g: HashSet<Box<Algo>> = HashSet::new();
    for x_var in x.get_all_variables() {
        //We create a base solver for each variables for variables that are not S(x)
        if x_var.contains('(') {
            continue;
        }
        g.insert(Box::new(Algo::base_solver(x_var)));
    }
    g
}

fn set_of_pair_of_algo(g: &HashSet<Box<Algo>>) -> HashSet<(Box<Algo>, Box<Algo>)> {
    let mut p: HashSet<(Box<Algo>, Box<Algo>)> = HashSet::new();
    g.clone().iter().enumerate().for_each(|(i, a1)| {
        g.clone().iter().enumerate().for_each(|(j, a2)| {
            if i < j {
                p.insert((a1.clone(), a2.clone()));
            }
        });
    });
    p
}

pub fn exhaustive_search(x: &mut Matrix, time_complexity: usize) -> HashSet<Box<Algo>> {
    //Set of base solvers
    let mut g: HashSet<Box<Algo>> = generate_all_base_solver(x);

    //Set of pair of algo
    let mut p: HashSet<(Box<Algo>, Box<Algo>)> = set_of_pair_of_algo(&g);

    // While g dont contains an algo with 20 variables
    while !p.is_empty() {
        //Take a pair of algo from p
        let a = p.iter().next().unwrap().clone();
        let (a1, a2) = p.take(&a).unwrap();

        let c = Box::new(Algo::fusion_two_algo(a1.clone(), a2.clone(), x));
        if c.get_time_complexity() <= time_complexity {
            update_queue(&mut g, &mut p, c);
        } else {
            println!("Time complexity reached")
        }
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

/// Keep all algo that are better or equal to c
fn keep_better(g: &mut HashSet<Box<Algo>>, c: &Box<Algo>) {
    g.retain(|elt| Some(Ordering::Less) != elt.compare1(&c));
}
///Keep all pair that are better or equal to c
fn keep_all_pair_that_are_better(p: &mut HashSet<(Box<Algo>, Box<Algo>)>, c: &Box<Algo>) {
    p.retain(|(a1, a2)| {
        !(a1.compare1(c) == Some(Ordering::Less) || a2.compare1(c) == Some(Ordering::Less))
    });
}
///Form new pairs with the algos such as the variables of one are not a subset of the other
fn add_new_pairs_no_doublon(
    g: &mut HashSet<Box<Algo>>,
    p: &mut HashSet<(Box<Algo>, Box<Algo>)>,
    c: &Box<Algo>,
) {
    let pairs = g.iter().flat_map(|a| {
        if c.get_all_variables().is_subset(&a.get_all_variables())
            || a.get_all_variables().is_subset(&c.get_all_variables())
        {
            None
        } else {
            Some((c.clone(), a.clone()))
        }
    });
    //Insert the new pairs in p
    p.extend(pairs);
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
        if aprim.compare1(&c) != Some(Ordering::Greater) {
            continue;
        } else {
            return;
        }
    }

    // Keep all algo that are better or equal to c
    keep_better(g, &c);

    // Add the new algo to the set
    g.insert(c.clone());

    // Keep all pair that are better or equal to c
    keep_all_pair_that_are_better(p, &c);

    //Form new pairs with the algos such as the variables of one are not a subset of the other
    add_new_pairs_no_doublon(g, p, &c);
}

#[cfg(test)]
mod test_exhaustive {

    use std::collections::HashMap;

    use crate::{parser::Parser, GlobalInfos};

    use super::*;

    #[test]
    fn test_exhaustive1() {
        let mut matrix = Matrix::from(vec![vec![1, 2, 3], vec![4, 3, 2], vec![4, 8, 2]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        matrix.set_vars_map(vars_maps.clone());
        exhaustive_search(&mut matrix, 50);
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
        let c1 = Algo::base_solver("A".to_string());
        let c2 = Algo::base_solver("B".to_string());
        let c3 = Algo::base_solver("C".to_string());
        let c4 = Algo::base_solver("D".to_string());
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

    #[test]
    fn test_generate_all_base_solver() {
        let system: &str = "equation_system/1r_3.txt";
        let mut globals: GlobalInfos = GlobalInfos::new(system.to_owned());
        let mut parser_mod = Parser::new(&globals);
        let mut matrix = parser_mod
            .parse_system(&mut globals)
            .expect("Error while parsing system");
        matrix.set_vars_map(parser_mod.vars_map);
        
        let g = generate_all_base_solver(&matrix);
        assert_eq!(g.len(), matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).count());
    }

    #[test]
    fn test_keep_better() {
        let system: &str = "equation_system/1r_3.txt";
        let mut globals: GlobalInfos = GlobalInfos::new(system.to_owned());
        let mut parser_mod = Parser::new(&globals);
        let mut matrix = parser_mod
            .parse_system(&mut globals)
            .expect("Error while parsing system");
        matrix.set_vars_map(parser_mod.vars_map);
        
        let g = generate_all_base_solver(&matrix);
        let c = Algo {
            vars: HashSet::from([matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).next().unwrap().clone()]),
            time: 100,
            memory: 10,
            nb_solutions: 10,
            son1: None,
            son2: None,
        };
        let mut gprim = g.clone();
        keep_better(&mut gprim, &Box::new(c.clone()));
        //C is really bad so it should not modify g
        assert_eq!(g, gprim);

        let c = Algo {
            vars: HashSet::from([matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).next().unwrap().clone()]),
            time: 0,
            memory: 10,
            nb_solutions: 10,
            son1: None,
            son2: None,
        };
        let mut gprim = g.clone();
        keep_better(&mut gprim, &Box::new(c.clone()));
        //C is better than one algo so g should be modified
        assert_eq!(gprim.len(),g.len() - 1);
    }

    #[test]
    fn test_keep_all_pair_that_are_better() {
        let system: &str = "equation_system/1r_3.txt";
        let mut globals: GlobalInfos = GlobalInfos::new(system.to_owned());
        let mut parser_mod = Parser::new(&globals);
        let mut matrix = parser_mod
            .parse_system(&mut globals)
            .expect("Error while parsing system");
        matrix.set_vars_map(parser_mod.vars_map);
        
        let g = generate_all_base_solver(&matrix);
        let c = Algo {
            vars: HashSet::from([matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).next().unwrap().clone()]),
            time: 100,
            memory: 10,
            nb_solutions: 10,
            son1: None,
            son2: None,
        };
        let p: HashSet<(Box<Algo>, Box<Algo>)> = set_of_pair_of_algo(&g);
        let mut pprim = p.clone();
        keep_all_pair_that_are_better(&mut pprim, &Box::new(c.clone()));
        //C is really bad so it should not modify p
        assert_eq!(p.len(), pprim.len());

        let c = Algo {
            vars: HashSet::from([matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).next().unwrap().clone()]),
            time: 0,
            memory: 10,
            nb_solutions: 10,
            son1: None,
            son2: None,
        };
        let mut pprim = p.clone();
        keep_all_pair_that_are_better(&mut pprim, &Box::new(c.clone()));
        //C is better than one algo so p should be modified
        assert_ne!(p.len(), pprim.len());
    }

    #[test]
    fn test_add_new_pairs_no_doublon() {
        let system: &str = "equation_system/1r_3.txt";
        let mut globals: GlobalInfos = GlobalInfos::new(system.to_owned());
        let mut parser_mod = Parser::new(&globals);
        let mut matrix = parser_mod
            .parse_system(&mut globals)
            .expect("Error while parsing system");
        matrix.set_vars_map(parser_mod.vars_map);
        
        let mut g = generate_all_base_solver(&matrix);
        let c = Algo {
            vars: HashSet::from([matrix.get_all_variables().iter().filter(|x| !x.contains("S(")).next().unwrap().clone()]),
            time: 100,
            memory: 10,
            nb_solutions: 10,
            son1: None,
            son2: None,
        };
        let p: HashSet<(Box<Algo>, Box<Algo>)> = set_of_pair_of_algo(&g);
        let mut pprim = p.clone();
        add_new_pairs_no_doublon(&mut g, &mut pprim, &Box::new(c.clone()));
        assert!(false);
    }
}
