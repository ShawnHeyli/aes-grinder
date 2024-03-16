use core::cmp::Ordering;

#[derive(PartialEq)]
pub struct Algo{
    time:i32,
    memory:i32,
    nb_solution:i32,
    son1:Option<Box<Algo>>,
    son2:Option<Box<Algo>>
}

impl PartialOrd for Algo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.time.cmp(&other.time).reverse())
    }
}


#[cfg(test)]
mod tests{
use super::Algo;
    #[test]
    fn compare_algo(){
        let algo_sad = Algo {
            time:100,
            memory:100,
            nb_solution:20,
            son1:None,
            son2:None
        };
        let algo_good = Algo {
            time:1,
            memory:1,
            nb_solution:1,
            son1:None,
            son2:None
        };

        assert!(algo_sad<algo_good);
    }
}