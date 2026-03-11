#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    values: Vec<i8>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, values: Vec<i8>) -> Result<Self, String> {
        if rows * cols != values.len() {
            return Err("matrix dimensions do not match value count".to_string());
        }
        Ok(Self { rows, cols, values })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> i8 {
        self.values[row * self.cols + col]
    }

    pub fn is_pm_one(&self) -> bool {
        self.values.iter().all(|value| *value == -1 || *value == 1)
    }

    pub fn gram_entry(&self, row_a: usize, row_b: usize) -> i32 {
        let mut total = 0_i32;
        for col in 0..self.cols {
            total += i32::from(self.get(row_a, col)) * i32::from(self.get(row_b, col));
        }
        total
    }

    pub fn is_hadamard(&self) -> bool {
        if self.rows != self.cols || !self.is_pm_one() {
            return false;
        }
        let n = self.rows as i32;
        for row_a in 0..self.rows {
            for row_b in 0..self.rows {
                let expected = if row_a == row_b { n } else { 0 };
                if self.gram_entry(row_a, row_b) != expected {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;

    #[test]
    fn sylvester_four_is_hadamard() {
        let values = vec![1, 1, 1, 1, 1, -1, 1, -1, 1, 1, -1, -1, 1, -1, -1, 1];
        let matrix = Matrix::new(4, 4, values).expect("matrix");
        assert!(matrix.is_hadamard());
    }
}
