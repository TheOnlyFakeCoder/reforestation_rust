use crate::formatting::format_g;

pub trait Printable {
    fn to_display_string(&self) -> String;
}

impl Printable for f64 {
    fn to_display_string(&self) -> String {
        format_g(*self, 6)
    }
}

impl Printable for i32 {
    fn to_display_string(&self) -> String {
        self.to_string()
    }
}

pub struct MatrixHandler;

impl MatrixHandler {
    pub fn new() -> Self {
        MatrixHandler
    }

    fn get_cof(matrix: &[Vec<f64>], p: usize, q: usize, n: usize) -> Vec<Vec<f64>> {
        let size = n.saturating_sub(1);
        let mut cof = vec![vec![0.0; size]; size];

        if n == 0 {
            return cof;
        }

        let mut i = 0;
        for row in 0..n {
            let mut j = 0;
            for col in 0..n {
                if row != p && col != q {
                    cof[i][j] = matrix[row][col];
                    j += 1;
                    if j == n - 1 {
                        j = 0;
                        i += 1;
                    }
                }
            }
        }

        cof
    }

    pub fn get_det(matrix: &[Vec<f64>], n: usize) -> f64 {
        if n == 1 {
            return matrix[0][0];
        }

        let mut det = 0.0;
        let mut sign = 1.0;

        for f in 0..n {
            let cof = Self::get_cof(matrix, 0, f, n);
            det += sign * matrix[0][f] * Self::get_det(&cof, n - 1);
            sign = -sign;
        }

        det
    }

    pub fn adjoint(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = matrix.len();
        let mut adj = vec![vec![0.0; n]; n];

        if n == 1 {
            adj[0][0] = 1.0;
            return adj;
        }

        for i in 0..n {
            for j in 0..n {
                let cof = Self::get_cof(matrix, i, j, n);
                let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                adj[j][i] = sign * Self::get_det(&cof, n - 1);
            }
        }

        adj
    }

    pub fn get_inverse(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let determinant = Self::get_det(matrix, matrix.len());
        let adj = Self::adjoint(matrix);
        adj.iter()
            .map(|row| row.iter().map(|value| value / determinant).collect())
            .collect()
    }

    pub fn inverse(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
        let determinant = Self::get_det(matrix, matrix.len());
        if determinant == 0.0 {
            println!("Singular matrix, can't find its inverse");
            return None;
        }

        let adj = Self::adjoint(matrix);
        Some(
            adj.iter()
                .map(|row| row.iter().map(|value| value / determinant).collect())
                .collect(),
        )
    }

    pub fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let rows = matrix.len();
        let cols = matrix.first().map_or(0, |row| row.len());

        let mut transposed = Vec::with_capacity(cols);
        for i in 0..cols {
            let mut new_row = Vec::with_capacity(rows);
            for j in 0..rows {
                new_row.push(matrix[j][i]);
            }
            transposed.push(new_row);
        }

        transposed
    }

    pub fn solve_linear(inverse_matrix: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
        let mut solution = vec![0.0; inverse_matrix.len()];
        for i in 0..inverse_matrix.len() {
            for j in 0..inverse_matrix.len() {
                solution[i] += b[j] * inverse_matrix[i][j];
            }
        }
        solution
    }

    pub fn print_matrix_elements<T: Printable>(matrix: &[Vec<T>]) {
        for row in matrix {
            print!("[ ");
            for value in row {
                print!("{} ", value.to_display_string());
            }
            println!("]");
        }
    }

    pub fn print_vector_elements(vector: &[f64]) {
        print!("[ ");
        for value in vector {
            print!("{} ", format_g(*value, 6));
        }
        println!("]");
    }
}

impl Default for MatrixHandler {
    fn default() -> Self {
        MatrixHandler
    }
}
