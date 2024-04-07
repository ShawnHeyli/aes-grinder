use std::collections::HashMap;

use num_integer::Integer;

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix {
    vars_map: HashMap<String, usize>, // Map the variable name to the column index
    rows: usize,
    cols: usize,
    data: Vec<usize>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            vars_map: HashMap::new(),
            rows,
            cols,
            data: vec![0; rows * cols],
        }
    }

    // Give a Vec of the row
    pub fn get_row(&self, row: usize) -> Vec<usize> {
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
    pub fn get_column(&self, column: usize) -> Vec<usize> {
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

    pub fn gaussian_elimination_inv(&mut self, modulus: usize) -> Matrix {
        for j in 0..self.cols {
            //Find the max
            let mut max = 0;
            let mut max_row = 0;
            for i in j..self.rows {
                if self[(i, j)] > max {
                    max = self[(i, j)];
                    max_row = i;
                }
            }
            for i in 0..self.cols {
                if self[(max_row, i)] != 0 && i == max_row {
                    //This is the pivot
                    //Set the pivot to one by multiplying the inverse of it in the field
                    //Use bigInt extended gcd to find the inverse
                    let pivot = self[(max_row, j)];
                    let mut inverse = (pivot as isize).extended_gcd(&(modulus as isize)).x;
                    while inverse < 0 {
                        inverse += modulus as isize;
                    }
                    let inverse = inverse as usize;
                    //Normalize the pivot line
                    for k in 0..self.cols {
                        self[(max_row, k)] = self[(max_row, k)] * inverse % modulus;
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
                            let a = self[(k, l)] as isize;
                            let b = factor as isize * self[(j, l)] as isize;
                            let mut ab = a - b;
                            while ab < 0 {
                                ab += modulus as isize;
                            }
                            let ab = ab as usize;
                            self[(k, l)] = ab % modulus;
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
                    let a = self[(i, k)] as isize;
                    let b = factor as isize * self[(j, k)] as isize;
                    let mut ab = a - b;
                    while ab < 0 {
                        ab += modulus as isize;
                    }
                    let ab = ab as usize;
                    self[(i, k)] = ab % modulus;
                }
            }
        }
        self.clone()
    }

    pub fn number_solutions(&self, vars: HashMap<String, u32>, modulus: usize) -> u32 {
        //Sort the columns by vars and non-vars

        //Apply gauss elimination on non-vars columns

        //Count the number of equations below

        todo!();
    }

    pub fn are_valid_values(&self, vars: &HashMap<String, u32>) -> bool {
        //Check in the equations where the vars appears if the values are possible
        todo!();
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
    type Output = usize;

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

impl From<Vec<Vec<usize>>> for Matrix {
    fn from(data: Vec<Vec<usize>>) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j]);
            }
        }

        matrix
    }
}

impl From<Vec<Vec<u32>>> for Matrix {
    fn from(data: Vec<Vec<u32>>) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j] as usize);
            }
        }

        matrix
    }
}

impl From<Vec<Vec<i32>>> for Matrix {
    fn from(data: Vec<Vec<i32>>) -> Self {
        let rows = data.len();
        let cols = data[0].len();
        let mut matrix = Matrix::new(rows, cols);
        matrix.data.clear();

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j] as usize);
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
        matrix[(0, 0)] = 1;
        assert_eq!(matrix[(0, 0)], 1);
    }

    #[test]
    fn test_index2() {
        let false_matrix = vec![vec![0; 2]; 2];
        let true_matrix = Matrix::new(2, 2);

        assert_eq!(true_matrix[(0, 0)], false_matrix[0][0]);
    }
    #[test]
    fn test_index3() {
        let mut false_matrix = vec![vec![0; 2]; 2];
        let mut true_matrix = Matrix::new(2, 2);

        false_matrix[1][1] = 4;
        true_matrix[(1, 1)] = 4;

        assert_eq!(true_matrix[(1, 1)], false_matrix[1][1]);
    }

    #[test]
    fn get_row() {
        let matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        let row = matrix.get_row(0);
        assert_eq!(row, vec![1, 2]);
    }

    #[test]
    fn get_column() {
        let matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        let column = matrix.get_column(0);
        assert_eq!(column, vec![1, 3]);
    }

    #[test]
    fn swap_columns() {
        let mut matrix = Matrix::from(vec![vec![1, 2], vec![3, 4]]);
        matrix.swap_columns(0, 1);
        let expected = Matrix::from(vec![vec![2, 1], vec![4, 3]]);
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
        let result = matrix.gaussian_elimination_inv(5);
        let expected = Matrix::from(vec![vec![1, 0], vec![0, 1]]);
        assert_eq!(result, expected);
    }
}
