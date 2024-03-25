//! Matrix abstraction
#[derive(Debug, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<usize>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![0; rows * cols],
        }
    }

    pub fn columns(&self) -> Vec<Vec<usize>> {
        let mut columns = vec![vec![0; self.rows]; self.cols];

        for i in 0..self.rows {
            for j in 0..self.cols {
                columns[j][i] = self.data[i * self.cols + j];
            }
        }

        columns
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..self.rows {
            for j in 0..self.cols {
                write!(f, "{} ", self.data[i * self.cols + j])?;
            }
            write!(f, "\n")?;
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

        for i in 0..rows {
            for j in 0..cols {
                matrix.data.push(data[i][j]);
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
}
