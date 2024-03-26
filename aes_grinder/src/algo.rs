use crate::matrix::Matrix;
use core::cmp::Ordering;

#[derive(PartialEq)]
pub struct Algo {
    matrix: Matrix,
    time: i32,
    memory: i32,
    nb_solution: i32,
    son1: Option<Box<Algo>>,
    son2: Option<Box<Algo>>,
}

impl PartialOrd for Algo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Some(self.time.cmp(&other.time).reverse())
        match  self.time.cmp(&other.time).reverse(){
            Ordering::Less => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Greater),
            Ordering::Equal => match self.memory.cmp(&other.memory).reverse() {
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
                Ordering::Equal => match self.nb_solution.cmp(&other.nb_solution).reverse() {
                    Ordering::Less => Some(Ordering::Less),
                    Ordering::Greater => Some(Ordering::Greater),
                    Ordering::Equal => Some(Ordering::Greater),
                }
            },
        } 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_algo() {
        let algo_sad = Algo {
            matrix: Matrix::new(0, 0),
            time: 100,
            memory: 100,
            nb_solution: 20,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            matrix: Matrix::new(0, 0),
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
            matrix: Matrix::new(0, 0),
            time: 2,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            matrix: Matrix::new(0, 0),
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
            matrix: Matrix::new(0, 0),
            time: 1,
            memory: 2,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            matrix: Matrix::new(0, 0),
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
            matrix: Matrix::new(0, 0),
            time: 2,
            memory: 2,
            nb_solution: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            matrix: Matrix::new(0, 0),
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
            matrix: Matrix::new(0, 0),
            time: 1,
            memory: 1,
            nb_solution: 2,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            matrix: Matrix::new(0, 0),
            time: 1,
            memory: 1,
            nb_solution: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }
}
