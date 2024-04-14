//! Struc Algo permettant de représenter des Algo
use std::{hash::Hash, vec};
use core::cmp::Ordering;
use std::{
    cmp::{max, min},
    collections::HashSet,
};

use crate::matrix::Matrix;
use std::hash::Hasher;

use std::fs::File;
use std::io::Write;
use std::process::Command;

#[derive(Eq, Clone, Debug)]
pub struct Algo {
    vars_val: Vec<String>,
    time: u32,
    memory: u32,
    nb_solutions: u32,
    son1: Option<Box<Algo>>,
    son2: Option<Box<Algo>>,
}

impl Hash for Algo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vars_val.hash(state);
        self.time.hash(state);
        self.memory.hash(state);
        self.nb_solutions.hash(state);
        self.son1.hash(state);
        self.son2.hash(state);
    }
}

impl PartialEq for Algo {
    fn eq(&self, other: &Self) -> bool {
        self.vars_val == other.vars_val
            && self.time == other.time
            && self.memory == other.memory
            && self.nb_solutions == other.nb_solutions
            && self.son1 == other.son1
            && self.son2 == other.son2
    }
    
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

///Implemtation de l'ordre partiel pour comparer deux algo entre eux
impl PartialOrd for Algo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if <Vec<std::string::String> as Clone>::clone(&other
            .vars_val)
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(&<Vec<std::string::String> as Clone>::clone(&self.vars_val).into_iter().collect::<HashSet<_>>())
        {
            match self.time.cmp(&other.time) {
                Ordering::Equal => match self.memory.cmp(&other.memory) {
                    Ordering::Equal => match self.nb_solutions.cmp(&other.nb_solutions) {
                        Ordering::Equal => Some(Ordering::Greater), // Tie breaker
                        Ordering::Greater => Some(Ordering::Less),
                        Ordering::Less => Some(Ordering::Greater),
                    },
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
                Ordering::Greater => Some(Ordering::Less),
                Ordering::Less => Some(Ordering::Greater),
            }
        } else {
            None
        }
    }
}

///Implementation de la struc algo
impl Algo {
    fn browse_algo_for_write (self, dot_file: &mut File, 
    cmpt: &mut u64) -> std::io::Result<()> {
        let mark_father = *cmpt;
        let mut mark_son_left = None;
        let mut mark_son_right = None;


        if mark_father == 0 {
            dot_file.write_all(
                format!("\tN{}[label=\"ROOT\"];\n", *cmpt).as_bytes()
            )?;
        } else {
            dot_file.write_all(
                format!("\tN{}[label=\"\"];\n", *cmpt).as_bytes()
            )?;
        }


        if let Some(son_left) = self.son1 {
            *cmpt += 1;
            mark_son_left = Some(*cmpt);
            son_left.browse_algo_for_write(dot_file, cmpt)?;
        }
        if let Some(son_right) = self.son2 {
            *cmpt += 1;
            mark_son_right = Some(*cmpt);
            son_right.browse_algo_for_write(dot_file, cmpt)?;
        }
        

        if !mark_son_left.is_none() {
            dot_file.write_all(
                format!("\tN{}", mark_father).as_bytes()
            )?;

            dot_file.write_all(
                format!(" -- N{}", mark_son_left.unwrap()).as_bytes()
            )?;

            dot_file.write_all(
                format!(";\n").as_bytes()
            )?;
        }

        if !mark_son_right.is_none() {
            dot_file.write_all(
                format!("\tN{}", mark_father).as_bytes()
            )?;

            dot_file.write_all(
                format!(" -- N{}", mark_son_right.unwrap()).as_bytes()
            )?;

            dot_file.write_all(
                format!(";\n").as_bytes()
            )?;
        }

        Ok(())
    }

    pub fn to_dot(self, filename: &str) -> std::io::Result<()>  {
        let mut file = File::create(filename)?;

        // Write data to the file
        file.write_all(
            format!("graph my_graph {{\n").as_bytes()
        )?;

        self.browse_algo_for_write (&mut file, & mut 0)?;

        file.write_all(
            format!("}}\n").as_bytes()
        )?;

        Ok(())
    }


    ///Constructeur d'un base solver
    pub fn base_solver(mut matrix: &mut Matrix, var: String) -> Algo {
        Algo {
            vars_val: vec![var.clone()],
            time: 8,
            memory: 8,
            nb_solutions: Matrix::number_solutions(&mut matrix, vec![var]),
            son1: None,
            son2: None,
        }
    }

    ///Fonction de fusion de deux algo
    pub fn fusion_two_algo(a1: Box<Algo>, a2: Box<Algo>) -> Algo {
        //Union of a1 vars_val and a2 vars_val
        let union_vars = a1
            .vars_val
            .clone()
            .into_iter()
            .chain(a2.vars_val.clone())
            .collect();

        //let nb_sol = Matrix::number_solutions(matrix, union_vars, modulus);
        let nb_sol = 1;
        Algo {
            vars_val: union_vars,
            //Compute the number of solutions
            nb_solutions: nb_sol,
            time: max(a1.time, max(a2.time, nb_sol)),
            memory: max(
                a1.memory,
                max(a2.memory, min(a1.nb_solutions, a2.nb_solutions)),
            ),
            son1: Some(a1),
            son2: Some(a2),
        }
    }

    /**
     * Definition of the comparision 1 (define in the paper)
     */
    pub fn compare1(&self, other: &Self) -> Option<Ordering> {
        if self.get_all_variables() == other.get_all_variables() {
            if(self.time <= other.time) {
                return Some(Ordering::Greater);
            }else {
                return Some(Ordering::Less);
            }
        }
        None
    }

    pub fn get_all_variables(&self) -> HashSet<String> {
        <Vec<std::string::String> as Clone>::clone(&self.vars_val).into_iter().collect()
    }

    pub fn get_time_complexity(&self) -> u32 {
        self.time
    }
}


///Test de l'implementation de la struct algo
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_algo() {
        let algo_sad = Algo {
            vars_val: vec!["x".to_string()],
            time: 100,
            memory: 100,
            nb_solutions: 20,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
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
            vars_val: vec!["x".to_string()],
            time: 2,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
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
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
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
            vars_val: vec!["x".to_string()],
            time: 2,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
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
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 2,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn to_dot_00() -> std::io::Result<()> {
        let algo_good = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        algo_good.to_dot("test/to_dot_00.dot")?;

        let status = Command::new("diff")
        .args(&["-q", "test/to_dot_00.dot", "test/to_dot_00_valid.dot"])
        .status()
        .expect("failed to execute diff");

        // Check the return code
        assert_eq!(Some(0), status.code());

        Ok(())
    }

    #[test]
    fn to_dot_01() -> std::io::Result<()> {
        let left = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let right = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let root = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: Some(Box::new(left)),
            son2: Some(Box::new(right)),
        };

        root.to_dot("test/to_dot_01.dot")?;

        let status = Command::new("diff")
        .args(&["-q", "test/to_dot_01.dot", "test/to_dot_01_valid.dot"])
        .status()
        .expect("failed to execute diff");

        // Check the return code
        assert_eq!(Some(0), status.code());

        Ok(())
    }

    #[test]
    fn to_dot_02() -> std::io::Result<()> {
        let c1_left = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let c1_right = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let c0_left = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: Some(Box::new(c1_left)),
            son2: Some(Box::new(c1_right)),
        };
        let c0_right = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let root = Algo {
            vars_val: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: Some(Box::new(c0_left)),
            son2: Some(Box::new(c0_right)),
        };

        root.to_dot("test/to_dot_02.dot")?;

        let status = Command::new("diff")
        .args(&["-q", "test/to_dot_02.dot", "test/to_dot_02_valid.dot"])
        .status()
        .expect("failed to execute diff");

        // Check the return code
        assert_eq!(Some(0), status.code());

        Ok(())
    }
}
