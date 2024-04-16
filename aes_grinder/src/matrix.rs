use crate::utils::{Invertible, Number};
use std::{cmp::max, cmp::min, collections::HashMap};

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix {
    vars_map: HashMap<String, usize>, // Map the variable name to the column index
    rows: usize,
    cols: usize,
    data: Vec<Number>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            vars_map: HashMap::new(),
            rows,
            cols,
            data: vec![0.into(); rows * cols],
        }
    }

    /// put argument vars to left of matrix
    fn sort_left(&mut self, vars: Vec<String>) {
        let mut swap_ndx: usize = 0;
        let vars_iter = vars.iter();

        for var in vars_iter {
            let ndx = self.vars_map.get(var).unwrap();
            self.swap_columns(swap_ndx, *ndx);

            assert_ne!(self.cols, swap_ndx);
            swap_ndx += 1;
        }
    }

    /// put argument vars to right of matrix
    fn sort_right(&mut self, vars: Vec<String>) {
        let mut swap_ndx: usize = self.cols - 1;
        let vars_iter = vars.iter();

        for var in vars_iter {
            let ndx = self.vars_map.get(var).unwrap();
            self.swap_columns(swap_ndx, *ndx);

            assert_ne!(0, swap_ndx);
            swap_ndx -= 1;
        }
    }

    pub fn new_from_vec(
        data: Vec<Vec<u32>>,
        _vars_map: HashMap<String, usize>,
        polynomial: u16,
    ) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();

        for i in 0..rows {
            for j in 0..cols {
                if data[i][j] >= 2u32.pow(16 - polynomial.leading_zeros()) {
                    panic!("Invalid number for the given polynomial");
                }
                matrix
                    .data
                    .push(Number::new(data[i][j].try_into().unwrap(), polynomial));
            }
        }

        matrix
    }

    pub fn get_row_number(&self) -> usize {
        self.rows
    }

    // Give a Vec of the row
    pub fn get_row(&self, row: usize) -> Vec<Number> {
        if row >= self.rows {
            panic!("Row index out of bounds");
        }

        let mut r = Vec::new();
        for i in 0..self.cols {
            r.push(self.data[row * self.cols + i]);
        }
        r
    }

    // Give a Vec of the column
    pub fn get_column(&self, column: usize) -> Vec<Number> {
        if column >= self.cols {
            panic!("Column index out of bounds");
        }

        let mut col = Vec::new();
        for i in 0..self.rows {
            col.push(self.data[i * self.cols + column]);
        }
        col
    }

    pub fn swap_columns(&mut self, col1: usize, col2: usize) {
        assert!(
            col1 < self.cols && col2 < self.cols,
            "Column index out of bounds"
        );

        for i in 0..self.rows {
            self.data.swap(i * self.cols + col1, i * self.cols + col2);
        }
        //Swap in vars_map
        for (_var, col) in self.vars_map.iter_mut() {
            if *col == col1 {
                *col = col2;
            } else if *col == col2 {
                *col = col1;
            }
        }
    }

    pub fn delete_row(&mut self, row: usize) {
        if row >= self.rows {
            panic!("Row index out of bounds");
        }

        // Remove the row
        let new_data = self
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| i / self.cols != row)
            .map(|(_, x)| *x)
            .collect();

        self.data = new_data;
        self.rows -= 1;
    }

    pub fn delete_column(&mut self, column: usize) {
        if column >= self.cols {
            panic!("Column index out of bounds");
        }

        // update the vars_map
        self.vars_map.retain(|_, v| *v != column);
        // update the column index in the vars_map after column
        for (_, v) in self.vars_map.iter_mut() {
            if *v > column {
                *v -= 1;
            }
        }

        // Remove the column
        let new_data = self
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| i % self.cols != column)
            .map(|(_, x)| *x)
            .collect();

        self.data = new_data;
        self.cols -= 1;
    }

    pub fn solve(&mut self) {
        for j in 0..min(self.cols, self.rows) {
            //Find the max
            let mut max: Number = 0.into();
            let mut max_row = 0;
            for i in j..self.rows {
                if self[(i, j)] > max {
                    max = self[(i, j)];
                    max_row = i;
                }
            }
            for i in 0..self.rows {
                if self[(i, j)] != 0.into() && i == max_row {
                    //This is the pivot
                    //Set the pivot to one by multiplying the inverse of it in the field
                    let pivot = self[(i, j)];
                    let inverse = pivot.invert();
                    //Normalize the pivot line
                    for k in 0..self.cols {
                        self[(i, k)] = self[(i, k)] * inverse;
                    }
                    //Swap the line
                    self.swap_lines(i, j);
                    //Set 0 under the pivot
                    for k in j + 1..self.rows {
                        let factor = self[(k, j)];
                        for l in 0..self.cols {
                            let a = self[(k, l)];
                            let b = factor * self[(j, l)];
                            let ab = a + b;
                            self[(k, l)] = ab;
                        }
                    }
                    break;
                }
            }
        }
        //Backward substitution
        for j in (0..min(self.cols, self.rows)).rev() {
            for i in (0..j).rev() {
                let factor = self[(i, j)];
                for k in 0..self.cols {
                    let a = self[(i, k)];
                    let b = factor * self[(j, k)];
                    let ab = a + b;
                    self[(i, k)] = ab;
                }
            }
        }
    }

    /// Perform gaussian elimination with inversion on the given variables and return the number of echelon rows
    pub fn solve_on(&mut self, vars: Vec<String>) -> usize {
        self.sort_left(vars.clone());
        let nb_echelon_rows = 0;
        for j in 0..vars.len() {
            //Find the max
            let mut max: Number = 0.into();
            let mut max_row = 0;
            for i in j..self.rows {
                if self[(i, j)] > max {
                    max = self[(i, j)];
                    max_row = i;
                }
            }
            for i in 0..self.rows {
                if self[(i, j)] != 0.into() && i == max_row {
                    //This is the pivot
                    //Set the pivot to one by multiplying the inverse of it in the field
                    let pivot = self[(i, j)];
                    let inverse = pivot.invert();
                    //Normalize the pivot line
                    for k in 0..self.cols {
                        self[(i, k)] = self[(i, k)] * inverse;
                    }
                    //Swap the line
                    self.swap_lines(i, j);
                    //Set 0 under the pivot
                    for k in j + 1..self.rows {
                        let factor = self[(k, j)];
                        for l in 0..self.cols {
                            let a = self[(k, l)];
                            let b = factor * self[(j, l)];
                            let ab = a + b;
                            self[(k, l)] = ab;
                        }
                    }
                    break;
                }
            }
        }
        //Backward substitution
        for j in (0..vars.len()).rev() {
            for j in (0..vars.len()).rev() {
                for i in (0..j).rev() {
                    let factor = self[(i, j)];
                    for k in 0..self.cols {
                        let a = self[(i, k)];
                        let b = factor * self[(j, k)];
                        let ab = a + b;
                        self[(i, k)] = ab;
                    }
                }
            }
        }
        nb_echelon_rows
    }

    fn swap_lines(&mut self, i: usize, j: usize) {
        println!("Swap lines {} and {}", i, j);
        assert!(
            i < self.rows && j < self.rows,
            "ERROR :: in swap_line :: out of bound {} {}",
            i,
            j
        );
        for k in 0..self.cols {
            let temp = self[(i, k)];
            self[(i, k)] = self[(j, k)];
            self[(j, k)] = temp;
        }
    }

    /// Perform row reduction to get row echelon form
    fn scale(&mut self) {
        for i in 0..self.rows {
            let row = self.get_row(i);
            let mut pivot = 0;
            for j in 0..self.cols {
                if row[j] != 0.into() {
                    pivot = j;
                    break;
                }
            }
            if pivot == 0 {
                continue;
            }
            for j in 0..self.rows {
                if j == i {
                    continue;
                }
                let inv = self[(j, pivot)].invert();
                for k in 0..self.cols {
                    self[(j, k)] = self[(j, k)] + inv * self[(i, k)];
                }
            }
        }
    }

    /// Row reduce the matrix on the given variables
    pub fn scale_on(&mut self, vars: Vec<String>) -> () {
        assert!(self.rows >= vars.len());
        self.sort_left(vars.clone());
        for j in 0..vars.len() {
            //Cols
            //Find the max
            let mut max: Number = 0.into();
            let mut max_row = 0;
            for i in j..self.rows {
                if self[(i, j)] > max {
                    max = self[(i, j)];
                    max_row = i;
                }
            }
            if max == 0.into() {
                continue;
            }
            //Swap the line
            self.swap_lines(max_row, j);
            //Normalize
            //Set the pivot to one by multiplying the inverse of it in the field
            let pivot = self[(j, j)];
            let inverse = pivot.invert();
            for k in 0..self.cols {
                self[(j, k)] = self[(j, k)] * inverse;
            }
            //Set 0 under the pivot
            for k in j + 1..self.rows {
                let factor = self[(k, j)];
                for l in 0..self.cols {
                    let a = self[(k, l)];
                    let b = factor * self[(j, l)];
                    let ab = a + b;
                    self[(k, l)] = ab;
                }
            }
        }
    }

    /**
     * Compute the number of solution of the system of equations for the given variables
     * Compute |vars| - dim(M(vars))
     */
    pub fn number_solutions(&mut self, vars: Vec<String>) -> usize {
        //Echelonner matrice sur les non vars
        //Compter nombre d'equation en bas (0 sous non vars, en dessous matrice echellonée)
        //C'est cette partie à gérer
        //Retourner |vars| - nombre d'equation en bas
        //Get variables from matrix that are not in vars
        let not_vars: Vec<String> = self
            .get_all_variables()
            .into_iter()
            .filter(|x| !vars.contains(x))
            .collect();
        self.solve_on(not_vars);

        //vieux code de samumu
        // vars.len() - (self.rows - echelon_rows)

        self.get_nb_ligne_zero_borded_from_bottom(vars.len())
    }

    fn get_nb_ligne_zero_borded_from_bottom(&self, nb_vars: usize) -> usize {
        let max = self.cols - nb_vars;
        let mut nb_ligne = 1;
        for i in (0..self.rows - 1).rev() {
            for j in 0..max {
                if self[(i, j)] != 0.into() {
                    return nb_ligne;
                }
            }
            nb_ligne += 1;
        }
        nb_ligne
    }

    fn get_matrix_generated_by(&self, vars: Vec<String>) -> Matrix {
        print!("get_matrix_generated_by in \n{}", self);
        let mut matrix = Matrix::new(self.rows, vars.len());
        for i in 0..self.rows {
            for j in 0..vars.len() {
                matrix[(i, j)] = self[(i, self.vars_map[&vars[j]])];
            }
        }
        print!("get_matrix_generated_by out \n{}", matrix);
        matrix
    }

    /// Compute the dimension of the solution space of the system of equations
    fn dimension_solution_space(&mut self) -> usize {
        self.scale();
        let r = self.count_no_zero_rows();
        println!(
            "ECHEC :  non_zero:{r} col : {:?}, row:{}",
            self.cols, self.rows
        );
        println!("MATRICE : \n{}", self);
        self.cols - r as usize
    }

    fn count_no_zero_rows(&self) -> u32 {
        let mut count = 0;
        for i in 0..self.rows {
            let row = self.get_row(i);
            let mut is_zero = true;
            for num in row {
                if num != 0.into() {
                    is_zero = false;
                    break;
                }
            }
            if !is_zero {
                count += 1;
            }
        }
        count
    }

    pub fn are_valid_values(&self, _vars: &HashMap<String, u32>) -> bool {
        //Check in the equations where the vars appears if the values are possible
        todo!();
    }

    ///Drop linear variable on the matrice, update the matrix self
    pub fn drop_linear_variable(&mut self) {
        println!(
            "Avant delete alone variable nb cols {}, nb rows {}",
            self.cols, self.rows
        );
        self.delete_alone_variable();
        println!(
            "Apres delete alone variable nb cols {}, nb rows {}",
            self.cols, self.rows
        );

        let mut has_been_update: bool = true;
        //tant que la matrice a ete mise a jour on continue d'eliminer les variables lineraires
        // let variable_of_max_rank: Vec<String> = self.get_variable_of_max_rank(1);
        // let mut variable_sboxed_max_rank_1 = get_variable_if_sboxed(&variable_of_max_rank);
        let mut variable_sboxed_max_rank_1 = get_variable_if_sboxed(&self.get_all_variables());
        println!(
            "tout les variable sboxed : {:?}",
            variable_sboxed_max_rank_1
        );
        while has_been_update {
            self.delete_empty_rows();
            self.delete_empty_colums();
            match variable_sboxed_max_rank_1.pop() {
                Some((x, sx)) => {
                    println!("VARIABLE ECHELONNE {x} et {sx}\n");
                    self.scale_on(vec![x, sx]);
                    //Here treat the case where the variable is linear
                }
                None => has_been_update = false,
            }
            println!("Apres gauss\n{}", self);
            // panic!();

            //     //selctionner une varibale dans les variables non traitées et de rang 1,
            //     //et qui a une varible en sbox aussi de rang1
            //     for (x, sx) in variable_sboxed_max_rank_1 {
            //         let x_index = self.vars_map.get(&x);
            //         let sx_index = self.vars_map.get(&sx);
            //         match (x_index, sx_index) {
            //             (Some(some_x), Some(some_y)) => {
            //                 let colum_x = matrix.get_column(*some_x);
            //                 let colum_sx = matrix.get_column(*some_y);
            //                 if colum_x == colum_sx {
            //                     todo!()
            //                 }
            //             }
            //             (_, _) => panic!(),
            //         }
            //     }
            //     todo!()

            //     compare les deux colonnes

            //     si elle sont egales on suprimme la ligne a 1 1 et les deux colonnes
        }
    }

    fn delete_alone_variable(&mut self) {
        let mut variables: Vec<String> = Vec::new();
        for (name, _) in &self.vars_map {
            for (str, _) in &self.vars_map {
                if name.contains(str) && name != str {
                    variables.push(str.to_string());
                    variables.push(name.to_string());
                }
            }
        }
        let mut variable_alone: Vec<String> = self.get_all_variables();
        variable_alone.retain(|s| !variables.contains(s));
        variable_alone
            .iter()
            .for_each(|s| self.remove_variable(s.to_string()));
    }

    ///Donne les indices des colonnes dans lequel le coef max est r
    fn get_col_of_max_rank(&self, r: usize) -> Vec<usize> {
        //trouve les colonnes de rang max r
        let mut col_rank: Vec<usize> = Vec::new();
        for i in 0..self.cols {
            let c = self.get_column(i);
            if c.iter().map(|number| number.get_value()).max().unwrap_or(0) <= r as u8 {
                col_rank.push(i);
            }
        }
        col_rank
    }

    ///Récupère les variables d'une colonne de rank max r
    fn get_variable_of_max_rank(&self, r: usize) -> Vec<String> {
        let my_col = self.get_col_of_max_rank(r);
        let mut variables: Vec<String> = Vec::new();
        for (str, col) in &self.vars_map {
            if my_col.contains(col) {
                variables.push(str.to_string());
            }
        }
        variables
    }

    /// Deletes all rows that are only made of 0 in place
    fn delete_empty_rows(&mut self) {
        let mut last_update = 0;
        while last_update < self.rows {
            let row = self.get_row(last_update);
            let mut is_zero = true;
            for num in row {
                if num.get_value() != 0 {
                    is_zero = false;
                    break;
                }
            }
            if is_zero {
                self.delete_row(last_update);
            } else {
                last_update += 1;
            }
        }
    }

    /// Deletes all columns that are only made of 0 in place
    fn delete_empty_colums(&mut self) {
        let mut last_update = 0;
        while last_update < self.cols {
            let column = self.get_column(last_update);
            let mut is_zero = true;
            for num in column {
                if num.get_value() != 0 {
                    is_zero = false;
                    break;
                }
            }
            if is_zero {
                self.delete_column(last_update);
            } else {
                last_update += 1;
            }
        }
    }

    ///Remove variables from string vec, update the matrix self
    fn remove_variable(&mut self, variables: String) {
        let col = match self.vars_map.get(&variables) {
            Some(c) => c,
            None => panic!("La Variable que l'on veut détruire n'existe pas"),
        };
        self.delete_column(*col);
    }

    ///Get all variable of the matrix
    pub fn get_all_variables(&self) -> Vec<String> {
        self.vars_map.keys().cloned().collect()
    }

    ///display variable names with their associated columns
    pub fn display_var_map(&self) {
        for (str, col) in &self.vars_map {
            println!("{} {}", str, col);
        }
    }

    ///Set vars map (we need to use a fonction because parser issue)
    pub fn set_vars_map(&mut self, vars_maps: HashMap<String, usize>) {
        self.vars_map = vars_maps;
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Print the vars_map in sorted order by values
        let vars_map = self.vars_map.clone();
        let mut vars_to_display: Vec<(String, usize)> = vars_map.into_iter().collect();
        vars_to_display.sort_by(|a, b| a.1.cmp(&b.1));
        for (str, col) in vars_to_display {
            writeln!(f, "{} {}", str, col)?;
        }

        // Print the matrix
        for i in 0..self.rows {
            for j in 0..self.cols {
                if self.data[i * self.cols + j] < 10.into() {
                    write!(f, "{}   ", self.data[i * self.cols + j])?;
                } else if self.data[i * self.cols + j] < 100.into() {
                    write!(f, "{}  ", self.data[i * self.cols + j])?;
                } else {
                    write!(f, "{} ", self.data[i * self.cols + j])?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Overload of []
impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = Number;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if index.0 >= self.rows || index.1 >= self.cols {
            panic!("Index out of bounds");
        }

        &self.data[index.0 * self.cols + index.1]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        if index.0 >= self.rows || index.1 >= self.cols {
            panic!("Index out of bounds");
        }

        &mut self.data[index.0 * self.cols + index.1]
    }
}

impl From<Vec<Vec<u8>>> for Matrix {
    fn from(data: Vec<Vec<u8>>) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();
        //Name the columns with alphabet
        for i in 0..rows {
            matrix.vars_map.insert(format!("X_{}", i), i);
        }

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j].into());
            }
        }

        matrix
    }
}

///retourne tout les variable S(x), x
pub fn get_variable_if_sboxed(variables: &Vec<String>) -> Vec<(String, String)> {
    let mut sboxed_variable: Vec<(String, String)> = vec![];
    for var in variables {
        for s in variables {
            if s.contains(var) && !s.eq(var) {
                sboxed_variable.push((var.to_string(), s.to_string()));
            }
        }
    }
    sboxed_variable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let mut matrix = Matrix::new(2, 2);
        matrix[(0, 0)] = 1.into();
        assert_eq!(matrix[(0, 0)], 1.into());
    }

    #[test]
    fn test_index2() {
        let false_matrix = vec![vec![0; 2]; 2];
        let true_matrix = Matrix::new(2, 2);

        assert_eq!(true_matrix[(0, 0)], false_matrix[0][0].into());
    }
    #[test]
    fn test_index3() {
        let mut false_matrix = vec![vec![0; 2]; 2];
        let mut true_matrix = Matrix::new(2, 2);

        false_matrix[1][1] = 4;
        true_matrix[(1, 1)] = 4.into();

        assert_eq!(true_matrix[(1, 1)], false_matrix[1][1].into());
    }

    #[test]
    fn get_row() {
        let matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        let row = matrix.get_row(0);
        assert_eq!(row, vec![1.into(), 2.into()]);
    }

    #[test]
    fn get_column() {
        let matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        let column = matrix.get_column(0);
        assert_eq!(column, vec![1.into(), 3.into()]);
    }

    #[test]
    fn delete_row() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        matrix.delete_row(0);
        let expected = Matrix::from(vec![vec![3, 4]]);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn delete_column_simple() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        matrix.delete_column(0);
        let expected = Matrix::from(vec![vec![2], vec![4]]);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn delete_column_vars_map() {
        let mut matrix = Matrix::from(vec![vec![0, 1, 1], vec![1, 0, 0], vec![0, 0, 1]]);
        matrix.vars_map.insert("a".to_string(), 0);
        matrix.vars_map.insert("b".to_string(), 1);
        matrix.vars_map.insert("c".to_string(), 2);
        matrix.delete_column(0);
        let mut expected = Matrix::from(vec![vec![1, 1], vec![0, 0], vec![0, 1]]);
        expected.vars_map = HashMap::new();
        expected.vars_map.insert("b".to_string(), 0);
        expected.vars_map.insert("c".to_string(), 1);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn delete_empty_rows() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![0, 0], vec![3, 4]]);
        println!("{}", matrix);
        matrix.delete_empty_rows();
        println!("{}", matrix);
        let expected = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn delete_empty_colums() {
        let mut matrix = Matrix::from(vec![vec![0, 2], vec![0, 4]]);
        matrix.delete_empty_colums();
        let expected = Matrix::from(vec![vec![2], vec![4]]);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn test_drop_linear_variable() {
        let mut matrix = Matrix::new(2, 2);
        matrix[(0, 0)] = 1.into();
        matrix[(0, 1)] = 1.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("X_0[0,0]".to_string(), 0);
        vars_maps.insert("S(X_0[0,0])".to_string(), 1);
        matrix.set_vars_map(vars_maps);

        let _expected = ["X_0[0,0]".to_string(), "S(X_0[0,0])".to_string()];
        println!("{}", matrix);
    }
    #[test]
    fn test_drop_linear_variable2() {
        let mut matrix = Matrix::new(4, 4);
        matrix[(0, 0)] = 1.into();
        matrix[(0, 1)] = 1.into();
        matrix[(2, 2)] = 4.into();
        matrix[(3, 3)] = 3.into();
        println!("{}", matrix);
        matrix.solve();
        println!("{}", matrix);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("X_0[0,0]".to_string(), 0);
        vars_maps.insert("S(X_0[0,0])".to_string(), 1);
        matrix.set_vars_map(vars_maps);
    }

    #[test]
    fn test_reduce_row() {
        // Construct matrix
        //| 1 2 8 7 |
        //| 6 7 2 5 |
        //| 9 5 1 2 |
        //| 6 4 8 1 |
        let mut matrix = Matrix::new(4, 4);
        matrix[(0, 0)] = 1.into();
        matrix[(0, 1)] = 2.into();
        matrix[(0, 2)] = 8.into();
        matrix[(0, 3)] = 7.into();
        matrix[(1, 0)] = 6.into();
        matrix[(1, 1)] = 7.into();
        matrix[(1, 2)] = 2.into();
        matrix[(1, 3)] = 5.into();
        matrix[(2, 0)] = 9.into();
        matrix[(2, 1)] = 5.into();
        matrix[(2, 2)] = 1.into();
        matrix[(2, 3)] = 2.into();
        matrix[(3, 0)] = 6.into();
        matrix[(3, 1)] = 4.into();
        matrix[(3, 2)] = 8.into();
        matrix[(3, 3)] = 1.into();
        matrix.set_vars_map(HashMap::from([
            (String::from("x"), 0),
            (String::from("y"), 1),
            (String::from("z"), 2),
            (String::from("k"), 3),
        ]));
        println!("{}", matrix);
        matrix.scale_on(vec![
            String::from("x"),
            String::from("y"),
            String::from("z"),
        ]);
        println!("{}", matrix);
    }

    #[test]
    fn test_get_col_of_max_rank() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 1.into();
        let test = matrix.get_col_of_max_rank(0);
        assert_eq!(test, vec![]);
        let test = matrix.get_col_of_max_rank(1);
        assert_eq!(test, vec![0, 1, 2]);
        let test = matrix.get_col_of_max_rank(2);
        assert_eq!(test, vec![0, 1, 2]);
        let test = matrix.get_col_of_max_rank(3);
        assert_eq!(test, vec![0, 1, 2]);
    }
    #[test]
    fn test_get_var_of_max_rank() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("W_0[0,0]".to_string(), 0);
        vars_maps.insert("S(X_0[1,1])".to_string(), 1);
        vars_maps.insert("X_0[1,1]".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        let mut test = matrix.get_variable_of_max_rank(0);
        test.sort();
        let mut vec: Vec<String> = vec![];
        vec.sort();
        assert_eq!(test, vec);
        let mut test = matrix.get_variable_of_max_rank(1);
        test.sort();
        let mut vec: Vec<String> = vec!["W_0[0,0]".to_string(), "S(X_0[1,1])".to_string()];
        vec.sort();
        assert_eq!(test, vec);
        let mut test = matrix.get_variable_of_max_rank(2);
        test.sort();
        let mut vec: Vec<String> = vec![
            "W_0[0,0]".to_string(),
            "S(X_0[1,1])".to_string(),
            "X_0[1,1]".to_string(),
        ];
        vec.sort();
        assert_eq!(test, vec);
        let mut test = matrix.get_variable_of_max_rank(3);
        test.sort();
        let mut vec: Vec<String> = vec![
            "W_0[0,0]".to_string(),
            "S(X_0[1,1])".to_string(),
            "X_0[1,1]".to_string(),
        ];
        vec.sort();
        assert_eq!(test, vec);
    }

    #[test]
    fn test_get_variable_if_sboxed() {
        let s: Vec<String> = vec!["X".to_string(), "S(X)".to_string(), "Y".to_string()];
        let sboxed = get_variable_if_sboxed(&s);
        let expect = vec![("X".to_string(), "S(X)".to_string())];
        print!("{:?}", s);
        print!("{:?}", expect);
        assert_eq!(sboxed, expect);
    }
    #[test]
    fn test_count_no_zero_rows() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();
        let z = matrix.count_no_zero_rows();
        assert_eq!(z, 3);
    }

    #[test]
    fn test_scale() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();

        let matrix2 = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();
        matrix.scale();
        println!("{}", matrix);
        println!("{}", matrix2);
        assert_eq!(matrix, matrix2);
    }

    #[test]
    fn test_delete_alone_variable() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("W_0[0,0]".to_string(), 0);
        vars_maps.insert("S(X_0[1,1])".to_string(), 1);
        vars_maps.insert("X_0[1,1]".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        matrix.delete_alone_variable();
        let mut m = matrix.get_all_variables();
        let mut expect = vec!["S(X_0[1,1])".to_string(), "X_0[1,1]".to_string()];
        m.sort();
        expect.sort();
        println!("variable de matrix = {:?}", m);
        println!("variable expect = {:?}", expect);

        assert_eq!(m, expect);
    }

    #[test]
    fn test_number_solutions() {
        //une solution
        let mut matrix = Matrix::from(vec![vec![1, 0], vec![0, 1]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        matrix.set_vars_map(vars_maps);
        let nb_sol = matrix.number_solutions(vec!["B".to_string()]);
        assert_eq!(1, nb_sol);
        let nb_sol = matrix.number_solutions(vec!["A".to_string()]);
    }
}

#[cfg(test)]
mod test_fn_solve {
    use super::*;

    #[test]
    fn test_solve_00() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        println!("{}", matrix);
        matrix.solve();
        let expected = Matrix::from(vec![vec![1, 0], vec![0, 1]]);
        println!("{}", matrix);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn test_solve_01() {
        let mut matrix = Matrix::from(vec![vec![1, 2, 3, 4], vec![4, 3, 2, 1]]);
        println!("UNSOLVED\n{}", matrix);

        matrix.solve();
        println!("SOLVED\n{}", matrix);

        assert_eq!(
            matrix.get_row(0),
            vec![1.into(), 0.into(), 0.into(), 0.into()]
        );
        assert_eq!(
            matrix.get_row(1),
            vec![0.into(), 1.into(), 0.into(), 0.into()]
        );
    }

    #[test]
    fn test_solve_02() {
        let mut matrix = Matrix::from(vec![vec![4, 4, 111], vec![4, 21, 250], vec![7, 8, 9]]);
        println!("UNSOLVED\n{}", matrix);

        matrix.solve();
        println!("SOLVED\n{}", matrix);

        assert_eq!(1, 2);
    }

    #[test]
    fn test_solve_03() {
        let mut matrix = Matrix::from(vec![vec![4, 4, 111], vec![4, 21, 250], vec![7, 8, 9]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        matrix.set_vars_map(vars_maps.clone());
        println!("matrice a : \n{}", matrix);
        let mut expected = Matrix::from(vec![vec![1, 0, 101], vec![0, 1, 56], vec![0, 0, 242]]);
        expected.set_vars_map(vars_maps.clone());

        matrix.solve();

        println!("matrice expected : \n{}", expected);
        println!("matrice obtenue : \n{}", matrix);
        assert_eq!(matrix, expected);
    }
}

#[cfg(test)]
mod test_fn_solve_on {
    use super::*;

    #[test]
    fn test_solve_on_00() {
        let mut matrix = Matrix::from(vec![vec![4, 4, 111], vec![4, 21, 250], vec![7, 8, 9]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        matrix.set_vars_map(vars_maps.clone());
        println!("matrice a : \n{}", matrix);
        let mut expected = Matrix::from(vec![vec![1, 0, 101], vec![0, 1, 56], vec![0, 0, 242]]);
        expected.set_vars_map(vars_maps.clone());

        matrix.solve_on(vec!["A".to_string(), "B".to_string()]);
        println!("matrice expected : \n{}", expected);
        println!("matrice obtenue : \n{}", matrix);
        assert_eq!(matrix, expected);
    }

    #[test]
    fn test_solve_on_01() {
        let mut matrix = Matrix::from(vec![vec![1, 2, 3, 4], vec![4, 3, 2, 1]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        vars_maps.insert("D".to_string(), 3);
        matrix.set_vars_map(vars_maps.clone());

        println!("matrice a : \n{}", matrix);
        //expected.set_vars_map(vars_maps.clone());

        matrix.solve_on(vec!["A".to_string(), "B".to_string()]);
        //println!("matrice expected : \n{}", expected);
        println!("matrice obtenue : \n{}", matrix);
        //assert_eq!(matrix, expected);

        assert_eq!(1, 2);
        //assert_eq!(matrix.get_row(0), vec![1.into(), 0.into(), 0.into(), 0.into()]);
        //assert_eq!(matrix.get_row(1), vec![0.into(), 1.into(), 0.into(), 0.into()]);
    }

    #[test]
    fn test_solve_on_02() {
        let mut matrix = Matrix::from(vec![vec![1, 2, 3, 4], vec![4, 3, 2, 1]]);
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        vars_maps.insert("D".to_string(), 3);
        matrix.set_vars_map(vars_maps.clone());

        println!("matrice a : \n{}", matrix);
        //expected.set_vars_map(vars_maps.clone());

        matrix.solve_on(vec!["A".to_string()]);
        //println!("matrice expected : \n{}", expected);
        println!("matrice obtenue : \n{}", matrix);
        //assert_eq!(matrix, expected);

        assert_eq!(1, 2);
        //assert_eq!(matrix.get_row(0), vec![1.into(), 0.into(), 0.into(), 0.into()]);
        //assert_eq!(matrix.get_row(1), vec![0.into(), 1.into(), 0.into(), 0.into()]);
    }
}

#[cfg(test)]
mod test_fn_swap {

    use super::*;

    #[test]
    fn swap_columns_00() {
        let mut matrix = Matrix::new(1, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        matrix.swap_columns(0, 2);

        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix.vars_map.get("z").unwrap(), &0);
        assert_eq!(matrix[(0, 0)], 2.into());
        assert_eq!(matrix[(0, 2)], 0.into());

        matrix.swap_columns(0, 1);

        assert_eq!(matrix.vars_map.get("y").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("z").unwrap(), &1);
        assert_eq!(matrix[(0, 0)], 1.into());
        assert_eq!(matrix[(0, 1)], 2.into());
    }

    #[test]
    fn swap_columns_01() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();
        matrix[(1, 0)] = 0.into();
        matrix[(1, 1)] = 1.into();
        matrix[(1, 2)] = 2.into();
        matrix[(2, 0)] = 0.into();
        matrix[(2, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        matrix.swap_columns(0, 2);

        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix.vars_map.get("z").unwrap(), &0);
        assert_eq!(matrix[(0, 0)], 2.into());
        assert_eq!(matrix[(1, 0)], 2.into());
        assert_eq!(matrix[(2, 0)], 2.into());
        assert_eq!(matrix[(0, 2)], 0.into());
        assert_eq!(matrix[(1, 2)], 0.into());
        assert_eq!(matrix[(2, 2)], 0.into());

        matrix.swap_columns(0, 1);

        assert_eq!(matrix.vars_map.get("y").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("z").unwrap(), &1);
        assert_eq!(matrix[(0, 0)], 1.into());
        assert_eq!(matrix[(1, 0)], 1.into());
        assert_eq!(matrix[(2, 0)], 1.into());
        assert_eq!(matrix[(0, 1)], 2.into());
        assert_eq!(matrix[(1, 1)], 2.into());
        assert_eq!(matrix[(2, 1)], 2.into());
    }

    #[test]
    fn test_number_solutions2() {
        let mut matrix = Matrix::from(vec![vec![1, 1, 0, 1], vec![0, 0, 1, 0], vec![0, 0, 0, 1]]);

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("A".to_string(), 0);
        vars_maps.insert("B".to_string(), 1);
        vars_maps.insert("C".to_string(), 2);
        vars_maps.insert("D".to_string(), 3);
        matrix.set_vars_map(vars_maps);
        println!(" m : {}", matrix);
        let nb_sol = matrix.number_solutions(vec!["C".to_string(), "D".to_string()]);
        print!("sol : {}", nb_sol);
        assert_eq!(2, nb_sol);
    }
}

#[cfg(test)]
mod test_fn_sort_left {
    use super::*;

    #[test]
    fn sort_left_00() {
        let mut matrix = Matrix::new(1, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        let string_lst = vec![String::from("z"), String::from("y")];
        matrix.sort_left(string_lst);

        assert_eq!(matrix.vars_map.get("z").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("y").unwrap(), &1);
        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix[(0, 0)], 2.into());
        assert_eq!(matrix[(0, 1)], 1.into());
        assert_eq!(matrix[(0, 2)], 0.into());
    }

    #[test]
    fn sort_left_01() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();
        matrix[(1, 0)] = 0.into();
        matrix[(1, 1)] = 1.into();
        matrix[(1, 2)] = 2.into();
        matrix[(2, 0)] = 0.into();
        matrix[(2, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        let string_lst = vec![String::from("y"), String::from("z")];
        matrix.sort_left(string_lst);

        assert_eq!(matrix.vars_map.get("y").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("z").unwrap(), &1);
        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix[(0, 0)], 1.into());
        assert_eq!(matrix[(1, 0)], 1.into());
        assert_eq!(matrix[(2, 0)], 1.into());
        assert_eq!(matrix[(0, 1)], 2.into());
        assert_eq!(matrix[(1, 1)], 2.into());
        assert_eq!(matrix[(2, 1)], 2.into());
        assert_eq!(matrix[(0, 2)], 0.into());
        assert_eq!(matrix[(1, 2)], 0.into());
        assert_eq!(matrix[(2, 2)], 0.into());
    }
}

#[cfg(test)]
mod test_fn_sort_right {
    use super::*;

    #[test]
    fn sort_right_00() {
        let mut matrix = Matrix::new(1, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        let string_lst = vec![String::from("x"), String::from("y")];
        matrix.sort_right(string_lst);

        assert_eq!(matrix.vars_map.get("z").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("y").unwrap(), &1);
        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix[(0, 0)], 2.into());
        assert_eq!(matrix[(0, 1)], 1.into());
        assert_eq!(matrix[(0, 2)], 0.into());
    }

    #[test]
    fn sort_right_01() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 0.into();
        matrix[(0, 1)] = 1.into();
        matrix[(0, 2)] = 2.into();
        matrix[(1, 0)] = 0.into();
        matrix[(1, 1)] = 1.into();
        matrix[(1, 2)] = 2.into();
        matrix[(2, 0)] = 0.into();
        matrix[(2, 1)] = 1.into();
        matrix[(2, 2)] = 2.into();

        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("x".to_string(), 0);
        vars_maps.insert("y".to_string(), 1);
        vars_maps.insert("z".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        let string_lst = vec![String::from("x"), String::from("y")];
        matrix.sort_right(string_lst);

        assert_eq!(matrix.vars_map.get("z").unwrap(), &0);
        assert_eq!(matrix.vars_map.get("y").unwrap(), &1);
        assert_eq!(matrix.vars_map.get("x").unwrap(), &2);
        assert_eq!(matrix[(0, 0)], 2.into());
        assert_eq!(matrix[(1, 0)], 2.into());
        assert_eq!(matrix[(2, 0)], 2.into());
        assert_eq!(matrix[(0, 1)], 1.into());
        assert_eq!(matrix[(1, 1)], 1.into());
        assert_eq!(matrix[(2, 1)], 1.into());
        assert_eq!(matrix[(0, 2)], 0.into());
        assert_eq!(matrix[(1, 2)], 0.into());
        assert_eq!(matrix[(2, 2)], 0.into());
    }
}
