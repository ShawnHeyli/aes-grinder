use std::{
    fmt::Display,
    ops::{Add, Mul},
};

trait SparseMatrix {
    fn new(shape: (usize, usize)) -> Self;
    fn from_dense(matrix: &[Vec<usize>]) -> Self;
    fn dissassemble(&self) -> Vec<Vec<usize>>;
}

#[derive(Debug)]
pub struct CooMatrix {
    row_indices: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<usize>,
    shape: (usize, usize),
}

impl CooMatrix {
    /// Get the matrix as a vector of triplets (row_index, col_index, value)
    pub fn get_triplets(&self) -> Vec<(usize, usize, usize)> {
        let mut triplets = Vec::new();
        for (i, &row_index) in self.row_indices.iter().enumerate() {
            triplets.push((row_index, self.col_indices[i], self.values[i]));
        }
        triplets
    }
}

impl SparseMatrix for CooMatrix {
    fn new(shape: (usize, usize)) -> Self {
        CooMatrix {
            row_indices: Vec::new(),
            col_indices: Vec::new(),
            values: Vec::new(),
            shape,
        }
    }

    /// Create a CooMatrix from a Vec<Vec<usize>>
    // Note: &[Vec<usize>] == Vec<Vec<usize>>
    fn from_dense(matrix: &[Vec<usize>]) -> Self {
        let mut coo_matrix = CooMatrix::new((matrix.len(), matrix[0].len()));
        for (i, row) in matrix.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                if value != usize::default() {
                    coo_matrix.row_indices.push(i);
                    coo_matrix.col_indices.push(j);
                    coo_matrix.values.push(value);
                }
            }
        }
        coo_matrix
    }

    /// Disassemble the CooMatrix into a Vec<Vec<usize>>
    fn dissassemble(&self) -> Vec<Vec<usize>> {
        let mut matrix = vec![vec![0; self.shape.1]; self.shape.0];
        for (i, &row_index) in self.row_indices.iter().enumerate() {
            matrix[row_index][self.col_indices[i]] = self.values[i];
        }
        matrix
    }
}

impl Display for CooMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.shape.0 {
            for j in 0..self.shape.1 {
                let mut is_exist = false;
                for k in 0..self.row_indices.len() {
                    if i == self.row_indices[k] && j == self.col_indices[k] {
                        write!(f, "{} ", self.values[k])?;
                        is_exist = true;
                        break;
                    }
                }
                if !is_exist {
                    write!(f, "0 ")?;
                }
            }
            // New line except for the last line
            if i != self.shape.0 - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Add for CooMatrix {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut result = Self::new(self.shape);
        result.row_indices = self.row_indices;
        result.col_indices = self.col_indices;
        result.values = self.values;
        for (i, &row_index) in rhs.row_indices.iter().enumerate() {
            let mut is_exist = false;
            for (j, &row_index_) in result.row_indices.iter().enumerate() {
                if row_index == row_index_ && rhs.col_indices[i] == result.col_indices[j] {
                    result.values[j] += rhs.values[i];
                    is_exist = true;
                    break;
                }
            }
            if !is_exist {
                result.row_indices.push(row_index);
                result.col_indices.push(rhs.col_indices[i]);
                result.values.push(rhs.values[i]);
            }
        }
        result
    }
}

// Multiplication is not commutative (m1 * m2 != m2 * m1)
impl Mul<CooMatrix> for CooMatrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result = Self::new((self.shape.0, rhs.shape.1));
        let mut row_indices = Vec::new();
        let mut col_indices = Vec::new();
        let mut values = Vec::new();
        for i in 0..self.row_indices.len() {
            for j in 0..rhs.row_indices.len() {
                if self.col_indices[i] == rhs.row_indices[j] {
                    row_indices.push(self.row_indices[i]);
                    col_indices.push(rhs.col_indices[j]);
                    values.push(self.values[i] * rhs.values[j]);
                }
            }
        }
        result.row_indices = row_indices;
        result.col_indices = col_indices;
        result.values = values;
        result
    }
}

impl Mul<usize> for CooMatrix {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self {
        let mut result = Self::new(self.shape);
        result.row_indices = self.row_indices;
        result.col_indices = self.col_indices;
        result.values = self.values.iter().map(|&x| x * rhs).collect();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coo_matrix() {
        let matrix = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let coo_matrix = CooMatrix::from_dense(&matrix);
        assert_eq!(coo_matrix.row_indices, vec![0, 1, 2]);
        assert_eq!(coo_matrix.col_indices, vec![0, 2, 3]);
        assert_eq!(coo_matrix.values, vec![1, 2, 3]);
        assert_eq!(coo_matrix.dissassemble(), matrix);
    }

    #[test]
    fn test_coo_matrix_get_triplets() {
        let matrix = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let coo_matrix = CooMatrix::from_dense(&matrix);
        assert_eq!(
            coo_matrix.get_triplets(),
            vec![(0, 0, 1), (1, 2, 2), (2, 3, 3)]
        );
    }

    #[test]
    fn test_coo_matrix_add() {
        let matrix1 = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let matrix2 = vec![vec![0, 0, 0, 4], vec![0, 0, 5, 0], vec![0, 0, 0, 6]];
        let coo_matrix1 = CooMatrix::from_dense(&matrix1);
        let coo_matrix2 = CooMatrix::from_dense(&matrix2);
        let result = coo_matrix1 + coo_matrix2;
        assert_eq!(result.row_indices, vec![0, 1, 2, 0]);
        assert_eq!(result.col_indices, vec![0, 2, 3, 3]);
        assert_eq!(result.values, vec![1, 7, 9, 4])
    }

    #[test]
    fn test_coo_matrix_mul1() {
        let matrix1 = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let matrix2 = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0], vec![4, 5, 6]];
        let coo_matrix1 = CooMatrix::from_dense(&matrix1);
        let coo_matrix2 = CooMatrix::from_dense(&matrix2);
        let result = coo_matrix1 * coo_matrix2;
        assert_eq!(result.row_indices, vec![2, 2, 2]);
        assert_eq!(result.col_indices, vec![0, 1, 2]);
        assert_eq!(result.values, vec![12, 15, 18]);
    }

    #[test]
    fn test_coo_matrix_mul2() {
        let matrix1 = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let matrix2 = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0], vec![4, 5, 6]];
        let coo_matrix1 = CooMatrix::from_dense(&matrix1);
        let coo_matrix2 = CooMatrix::from_dense(&matrix2);
        let result = coo_matrix2 * coo_matrix1;
        assert_eq!(result.row_indices, vec![3, 3, 3]);
        assert_eq!(result.col_indices, vec![0, 2, 3]);
        assert_eq!(result.values, vec![4, 10, 18]);
    }

    #[test]
    fn test_coo_matrix_mul_scalar() {
        let matrix = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let coo_matrix = CooMatrix::from_dense(&matrix);
        let result = coo_matrix * 2;
        assert_eq!(result.row_indices, vec![0, 1, 2]);
        assert_eq!(result.col_indices, vec![0, 2, 3]);
        assert_eq!(result.values, vec![2, 4, 6]);
    }

    #[test]
    fn test_coo_matrix_display() {
        let matrix = vec![vec![1, 0, 0, 0], vec![0, 0, 2, 0], vec![0, 0, 0, 3]];
        let coo_matrix = CooMatrix::from_dense(&matrix);
        assert_eq!(format!("{}", coo_matrix), "1 0 0 0 \n0 0 2 0 \n0 0 0 3 ");
    }
}
