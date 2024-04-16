//! Struc Algo permettant de représenter des Algo
use crate::matrix::Matrix;
use core::cmp::Ordering;
use std::fs::File;
use std::hash::Hasher;
use std::io::Write;
use std::{
    cmp::{max, min},
    collections::HashSet,
};
use std::{hash::Hash, vec};

#[derive(Eq, Clone, Debug)]
pub struct Algo {
    vars: Vec<String>,
    time: usize,
    memory: u32,
    nb_solutions: usize,
    son1: Option<Box<Algo>>,
    son2: Option<Box<Algo>>,
}

impl Hash for Algo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vars.hash(state);
        self.time.hash(state);
        self.memory.hash(state);
        self.nb_solutions.hash(state);
        self.son1.hash(state);
        self.son2.hash(state);
    }
}

impl PartialEq for Algo {
    fn eq(&self, other: &Self) -> bool {
        self.vars == other.vars
            && self.time == other.time
            && self.memory == other.memory
            && self.nb_solutions == other.nb_solutions
            && self.son1 == other.son1
            && self.son2 == other.son2
    }
}

///Implemtation de l'ordre partiel pour comparer deux algo entre eux
impl PartialOrd for Algo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if <Vec<std::string::String> as Clone>::clone(&other.vars)
            .into_iter()
            .collect::<HashSet<_>>()
            .is_subset(
                &<Vec<std::string::String> as Clone>::clone(&self.vars)
                    .into_iter()
                    .collect::<HashSet<_>>(),
            )
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
    fn browse_algo_for_write(&self, dot_file: &mut File, cmpt: &mut u64) -> std::io::Result<()> {
        let mark_father = *cmpt;
        let mut mark_son_left = None;
        let mut mark_son_right = None;

        if mark_father == 0 {
            dot_file.write_all(
                format!(
                    "\t{}[style=\"filled\" label=\"nb_sol = {}\" color=\"firebrick1\"];\n",
                    *cmpt, self.nb_solutions
                )
                .as_bytes(),
            )?;
        } else {
            if self.vars.len() == 1 {
                dot_file.write_all(
                    format!(
                        "\t{}[style=\"filled\" label=\"{}\nnb_sol = {}\" color=\"chartreuse\"];\n",
                        *cmpt, self.vars[0], self.nb_solutions
                    )
                    .as_bytes(),
                )?;
            } else {
                dot_file.write_all(
                    format!("\t{}[label=\"nb_sol = {}\"];\n", *cmpt, self.nb_solutions).as_bytes(),
                )?;
            }
        }

        if let Some(son_left) = &self.son1 {
            *cmpt += 1;
            mark_son_left = Some(*cmpt);
            son_left.browse_algo_for_write(dot_file, cmpt)?;
        }
        if let Some(son_right) = &self.son2 {
            *cmpt += 1;
            mark_son_right = Some(*cmpt);
            son_right.browse_algo_for_write(dot_file, cmpt)?;
        }

        if mark_son_left.is_some() || mark_son_right.is_some() {
            dot_file.write_all(format!("\t{} -> {{", mark_father).as_bytes())?;

            if mark_son_left.is_some() {
                dot_file.write_all(format!(" {}", mark_son_left.unwrap()).as_bytes())?;
            }
            if mark_son_right.is_some() {
                dot_file.write_all(format!(" {}", mark_son_right.unwrap()).as_bytes())?;
            }

            dot_file.write_all("}\n".to_string().as_bytes())?;
        }

        Ok(())
    }

    /// Print into filename the dot corresponding to self Algo
    pub fn to_dot(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;

        // Write data to the file
        file.write_all(format!("digraph {{\n").as_bytes())?;

        self.browse_algo_for_write(&mut file, &mut 0)?;

        file.write_all(format!("}}\n").as_bytes())?;

        Ok(())
    }

    ///Constructeur d'un base solver
    pub fn base_solver(mut matrix: &mut Matrix, var: String) -> Algo {
        Algo {
            vars: vec![var],
            time: 8,
            memory: 8,
            nb_solutions: 256,
            son1: None,
            son2: None,
        }
    }

    ///Fonction de fusion de deux algo
    pub fn fusion_two_algo(a1: Box<Algo>, a2: Box<Algo>, mut matrix: &mut Matrix) -> Algo {
        let mut union_vars: Vec<String> = a1.vars.clone();
        let union_vars2: Vec<String> = a2.vars.clone();
        union_vars.extend(union_vars2);
        union_vars.dedup();
        let union_varstmp = union_vars.clone();

        let nb_sol = Matrix::number_solutions(matrix, union_vars.clone());
        let alg = Algo {
            vars: union_vars,
            //Compute the number of solutions
            nb_solutions: nb_sol,
            time: max(a1.time, max(a2.time, nb_sol)),
            memory: max(
                a1.memory,
                max(
                    a2.memory,
                    min(
                        a1.nb_solutions.try_into().unwrap(),
                        a2.nb_solutions.try_into().unwrap(),
                    ),
                ),
            ),
            son1: Some(a1),
            son2: Some(a2),
        };
        let mut h = std::hash::DefaultHasher::new();
        alg.hash(&mut h);

        let name = format!("/tmp/algo{}", h.finish());
        println!(
            "fusion {:?} cardinal algo : {}",
            alg.to_dot(&name),
            union_varstmp.len()
        );
        alg
    }

    /// Compares two algorithms if they have the same variables, the one with the smallest time is better
    /// Corresponds to comparaison1 in the paper
    pub fn compare1(&self, other: &Self) -> Option<Ordering> {
        let mut vec1 = self.vars.clone();
        let mut vec2 = other.vars.clone();

        vec1.sort();
        vec2.sort();

        if vec1 == vec2 {
            if vec1 <= vec2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        } else {
            None
        }
    }

    pub fn get_all_variables(&self) -> HashSet<String> {
        self.vars.iter().cloned().collect::<HashSet<String>>()
    }

    pub fn get_time_complexity(&self) -> usize {
        self.time
    }
}

///Test de l'implementation de la struct algo
#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn compare_algo() {
        let algo_sad = Algo {
            vars: vec!["x".to_string()],
            time: 100,
            memory: 100,
            nb_solutions: 20,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars: vec!["x".to_string()],
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
            vars: vec!["x".to_string()],
            time: 2,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars: vec!["x".to_string()],
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
            vars: vec!["x".to_string()],
            time: 1,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars: vec!["x".to_string()],
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
            vars: vec!["x".to_string()],
            time: 2,
            memory: 2,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars: vec!["x".to_string()],
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
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 2,
            son1: None,
            son2: None,
        };
        let algo_good = Algo {
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };

        assert!(algo_sad < algo_good);
    }

    #[test]
    fn test_number_solutions() {
        println!("Test number solutions");
        let mut matrix = Matrix::from(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]);
        let algo = Algo::base_solver(&mut matrix, "X_1".to_string());
        println!("After num sol\n{}", matrix);
        assert_eq!(1, algo.nb_solutions);
    }

    #[test]
    fn to_dot_00() -> std::io::Result<()> {
        let algo_good = Algo {
            vars: vec!["x".to_string()],
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
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let right = Algo {
            vars: vec!["x".to_string(), "y".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let root = Algo {
            vars: vec!["x".to_string(), "y".to_string()],
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
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let c1_right = Algo {
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let c0_left = Algo {
            vars: vec!["x".to_string(), "y".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: Some(Box::new(c1_left)),
            son2: Some(Box::new(c1_right)),
        };
        let c0_right = Algo {
            vars: vec!["x".to_string()],
            time: 1,
            memory: 1,
            nb_solutions: 1,
            son1: None,
            son2: None,
        };
        let root = Algo {
            vars: vec!["x".to_string()],
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
