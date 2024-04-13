use std::collections::HashMap;
use crate::utils::{Invertible, Number};
use std::collections::HashMap;

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

    pub fn new_from_vec(data: Vec<Vec<u32>>, vars_map: HashMap<String, usize>, polynomial: u16) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();

        for i in 0..rows {
            for j in 0..cols {
                if data[i][j] >= 2u32.pow((16 - polynomial.leading_zeros()) as u32) {
                    panic!("Invalid number for the given polynomial");
                }
                matrix.data.push(Number::new(data[i][j].try_into().unwrap(), polynomial));
            }
        }

        matrix
    }

    pub fn get_row_number (&self) -> usize {
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
        if col1 >= self.cols || col2 >= self.cols {
            panic!("Column index out of bounds");
        }

        for i in 0..self.rows {
            self.data.swap(i * self.cols + col1, i * self.cols + col2);
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

    pub fn gaussian_elimination_inv(&mut self) -> Matrix {
        for j in 0..self.cols {
            //Find the max
            let mut max: Number = 0.into();
            let mut max_row = 0;
            for i in j..self.rows {
                if self[(i, j)] > max {
                    max = self[(i, j)];
                    max_row = i;
                }
            }
            for i in 0..self.cols {
                if self[(max_row, i)] != 0.into() && i == max_row {
                    //This is the pivot
                    //Set the pivot to one by multiplying the inverse of it in the field
                    //Use bigInt extended gcd to find the inverse
                    let pivot = self[(max_row, j)];
                    let mut inverse = pivot.invert();
                    //Normalize the pivot line
                    for k in 0..self.cols {
                        self[(max_row, k)] = self[(max_row, k)] * inverse;
                    }
                    //Swap the line
                    for k in 0..self.rows {
                        let temp = self[(j, k)];
                        self[(j, k)] = self[(max_row, k)];
                        self[(max_row, k)] = temp;
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
                    break;
                }
            }
        }
        //Backward substitution
        for j in (0..self.cols).rev() {
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
        self.clone()
    }

    pub fn number_solutions(&mut self, _vars: Vec<String>) -> u32 {
        //Sort the columns by vars and non-vars

        //Apply gauss elimination on non-vars columns
        self.gaussian_elimination_inv();
        //Count the number of equations below
        
        todo!();
    }

    pub fn are_valid_values(&self, _vars: &HashMap<String, u32>) -> bool {
        //Check in the equations where the vars appears if the values are possible
        todo!();
    }

    ///Get non linear variable, return variables as a string vector
    pub fn get_non_linear_variable(&self) -> Vec<String> {
        let mut non_linear_variables: Vec<String> = vec![];
        for (var, _) in &self.vars_map {
            if var.contains('(') {
                let true_var = var.clone();
                let var: Vec<_> = var.split(['(', ')']).collect();
                non_linear_variables.push(true_var);
                non_linear_variables.push(var[1].to_string());
            }
        }
        non_linear_variables
    }

    ///Drop linear variable on the matrice, update the matrix self
    pub fn drop_linear_variable(&mut self) {
        let all_variables = self.get_all_variables();
        let non_linear_variables = self.get_non_linear_variable();
        let linear_variables: Vec<String> = all_variables
            .into_iter()
            .filter(|x| !non_linear_variables.contains(x))
            .collect();

        for var in linear_variables {
            self.remove_variable(var);
        }

        //Detruire la ligne vide si retirer la variable met une equation a zero
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
        // Print the vars_map
        for (k, v) in &self.vars_map {
            writeln!(f, "{}: {}", k, v)?;
        }

        // Print the matrix
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{} ", self.data[i * self.cols + j])?;
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

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j].into());
            }
        }

        matrix
    }
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
    fn swap_columns() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        matrix.swap_columns(0, 1);
        let expected = Matrix::from(vec![vec![2, 1], vec![4, 3]]);
        assert_eq!(matrix, expected);
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
    fn test_gaussian_elimination_inv() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        let result = matrix.gaussian_elimination_inv();
        let expected = Matrix::from(vec![vec![1, 0], vec![0, 1]]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_non_get_linear_variable() {
        let mut matrix = Matrix::new(6, 7);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 1.into();
        matrix[(3, 3)] = 2.into();
        matrix[(3, 0)] = 1.into();
        matrix[(3, 4)] = 1.into();
        matrix[(4, 5)] = 1.into();
        matrix[(5, 6)] = 1.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("S(X_0[3,3])".to_string(), 6);
        vars_maps.insert("S(X_0[1,1])".to_string(), 4);
        vars_maps.insert("W_0[0,0]".to_string(), 0);
        vars_maps.insert("S(X_0[2,2])".to_string(), 5);
        vars_maps.insert("K_1[0,0]".to_string(), 1);
        vars_maps.insert("C[0,0]".to_string(), 2);
        vars_maps.insert("S(X_0[0,0])".to_string(), 3);
        matrix.set_vars_map(vars_maps);

        let mut expected = vec![
            "S(X_0[0,0])".to_string(),
            "S(X_0[1,1])".to_string(),
            "S(X_0[2,2])".to_string(),
            "S(X_0[3,3])".to_string(),
            "X_0[3,3]".to_string(),
            "X_0[1,1]".to_string(),
            "X_0[2,2]".to_string(),
            "X_0[0,0]".to_string(),
        ];
        expected.sort();
        let mut non_linear = matrix.get_non_linear_variable();
        non_linear.sort();

        assert_eq!(non_linear, expected);
    }

    #[test]
    fn test_drop_linear_variable() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 1.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("S(X_0[1,1])".to_string(), 0);
        vars_maps.insert("X_0[1,1]".to_string(), 1);
        vars_maps.insert("W_0[0,0]".to_string(), 2);
        matrix.set_vars_map(vars_maps);

        println!("{}", matrix);
        matrix.drop_linear_variable();
        print!("{}", matrix);

        //Detruire la ligne vide si retirer la variable met une equation a zero
    }

    #[test]
    fn test_drop_linear_variable2() {
        let mut matrix = Matrix::new(3, 3);
        matrix[(0, 0)] = 1.into();
        matrix[(1, 1)] = 1.into();
        matrix[(2, 2)] = 1.into();
        let mut vars_maps: HashMap<String, usize> = HashMap::new();
        vars_maps.insert("S(X_0[1,1])".to_string(), 1);
        vars_maps.insert("X_0[1,1]".to_string(), 2);
        vars_maps.insert("W_0[0,0]".to_string(), 0);
        matrix.set_vars_map(vars_maps);

        println!("{}", matrix);
        matrix.drop_linear_variable();
        print!("{}", matrix);
        //Detruire la ligne vide si retirer la variable met une equation a zero
    }
}
