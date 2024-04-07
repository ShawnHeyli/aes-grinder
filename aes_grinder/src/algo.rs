//! Struc Algo permettant de représenter des Algo


use core::cmp::Ordering;
use std::{cmp::{max, min}, collections::{HashMap, HashSet}};

use crate::matrix::Matrix;

//La struct algo
#[derive(PartialEq)]
pub struct Algo {
    vars_val: HashMap<String, u32>,
    time: u32,
    memory: u32,
    nb_solutions: u32,
    son1: Option<Box<Algo>>,
    son2: Option<Box<Algo>>,
}

///Implemtation de l'ordre partiel pour comparer deux algo entre eux
impl PartialOrd for Algo {
    //TODO Verifier la comparaison inferieur + equal
    // faire des test sur des égalité
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other.vars_val.keys().collect::<HashSet<_>>().is_subset(&self.vars_val.keys().collect::<HashSet<_>>()) {
            if self.time <= other.time {
                if self.memory <= other.memory {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            } else {
                return Some(Ordering::Less);
            }
        } else {
            return None;
        }
    }
}

///Implementation de la struc algo
impl Algo {

    ///Constructeur d'un base solver
    pub fn base_solver(matrix: &Matrix, var: String, modulus: usize) -> Algo {
        //Choose value for var that is a valid value in the matrix
        let mut vars = HashMap::new();
        for x in 0..256 {
            vars.insert(var.clone(), x);
            if matrix.are_valid_values(&vars) {
                break;
            }
        }
        Algo {
            vars_val: vars.clone(),
            time: 8,
            memory: 8,
            nb_solutions: Matrix::number_solutions(matrix, vars.clone(), modulus),
            son1: None,
            son2: None,
        }
    }


    ///Fonction de fusion de deux algo
    pub fn fusion_two_algo(a1: Box<Algo>, a2: Box<Algo>) -> Algo {
        //Union of a1 vars_val and a2 vars_val
        let union_vars = a1.vars_val.clone().into_iter().chain(a2.vars_val.clone()).collect();

        //let nb_sol = Matrix::number_solutions(matrix, union_vars, modulus);
        let nb_sol=1;
        Algo {
            vars_val: union_vars,
            //Compute the number of solutions
            nb_solutions: nb_sol,
            time: max(a1.time, max(a2.time, nb_sol)),
            memory: max(a1.memory, max(a2.memory, min(a1.nb_solutions, a2.nb_solutions))),
            son1: Some(a1),
            son2: Some(a2),
        }
    }

}

///Test de l'implementation de la struct algo
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_algo() {
        let algo_sad = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 100,
            memory: 100,
            nb_solutions: 20,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
    #[test]
    fn compare_algo_time() {
        let algo_sad = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 2,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn compare_algo_memory_for_same_time() {
        let algo_sad = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
    #[test]
    fn compare_algo_time_and_memory() {
        let algo_sad = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 2,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn compare_algo_nb_solution() {
        let algo_sad = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 2,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
}
