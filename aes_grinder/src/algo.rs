use core::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(PartialEq)]
pub struct Algo {
    vars_map: HashMap<String, u32>,
    time: i32,
    memory: i32,
    nb_solution: i32,
    son1: Option<Box<Algo>>,
    son2: Option<Box<Algo>>,
}

impl PartialOrd for Algo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other.vars_map.keys().collect::<HashSet<_>>().is_subset(&self.vars_map.keys().collect::<HashSet<_>>()) {
            if self.time < other.time {
                if self.memory < other.memory {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_algo() {
        let algo_sad = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 100,
            memory: 100,
            nb_solution: 20,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
    #[test]
    fn compare_algo_time() {
        let algo_sad = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 2,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn compare_algo_memory_for_same_time() {
        let algo_sad = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 2,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
    #[test]
    fn compare_algo_time_and_memory() {
        let algo_sad = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 2,
            memory: 2,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn compare_algo_nb_solution() {
        let algo_sad = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 2,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_map: HashMap::from([(String::from("x"), 1)]),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
}
