use crate::matrix::Matrix;
use std::collections::HashSet;

fn exhaustive_search<E>(x: Matrix, time_complexity: i32) -> Vec<Algo> {
    //Set of base solvers
    let mut g:HashSet<Algo> = HashSet::new();
    for xVar in x.vars_map.keys() {
        //We create a base solver for each variable présente
        gVar.insert(Algo::base_solver(&x, xVar, 256));
    }
    //Set of pair of algo
    let mut p:HashSet<(Algo, Algo)> = HashSet::new();
    
    for (i, item1) in g.iter().enumerate() {
            for item2 in g.iter().enumerate() {
            p.insert((item1, item2));
        }
    }
    

    while !p.is_empty() {
        let (a1, a2) = p.pop();
        let c = fusion_two_algo(a1, a2);

        if c.time <= time_complexity {
            let (new_g, new_p) = update_queue(g, p, c);
            g = new_g;
            p = new_p;
        }
    }
    
    g
}

fn update_queue(g:HashSet<Algo>, p:HashSet<(Algo, Algo)>, c:Algo) -> (HashSet<Algo>, HashSet<(Algo, Algo)>){
    (GPrime, PPrime)
} 