#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    values: Vec<i8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HadamardCheck {
    pub is_hadamard: bool,
    pub off_diagonal_pairs_checked: usize,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, values: Vec<i8>) -> Result<Self, String> {
        let value_count = rows
            .checked_mul(cols)
            .ok_or_else(|| "matrix dimensions overflow the value count".to_string())?;
        if value_count != values.len() {
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
        self.check_hadamard().is_hadamard
    }

    pub fn check_hadamard(&self) -> HadamardCheck {
        if self.rows == 0 || self.rows != self.cols || !self.is_pm_one() {
            return HadamardCheck {
                is_hadamard: false,
                off_diagonal_pairs_checked: 0,
            };
        }

        // For +/-1 rows, a zero dot product is equivalent to Hamming distance n/2.
        // Packing signs into words keeps the check exact while avoiding a scalar dot
        // product for every pair of rows.
        let words_per_row = (self.cols - 1) / 64 + 1;
        let mut packed_rows = vec![0_u64; self.rows * words_per_row];
        for row in 0..self.rows {
            for col in 0..self.cols {
                if self.get(row, col) == 1 {
                    packed_rows[row * words_per_row + col / 64] |= 1_u64 << (col % 64);
                }
            }
        }

        let mut checked = 0_usize;
        for left in 0..self.rows {
            let left_start = left * words_per_row;
            for right in (left + 1)..self.rows {
                let right_start = right * words_per_row;
                let distance = (0..words_per_row)
                    .map(|word| {
                        (packed_rows[left_start + word] ^ packed_rows[right_start + word])
                            .count_ones() as usize
                    })
                    .sum::<usize>();
                checked += 1;
                if distance.checked_mul(2) != Some(self.cols) {
                    return HadamardCheck {
                        is_hadamard: false,
                        off_diagonal_pairs_checked: checked,
                    };
                }
            }
        }

        HadamardCheck {
            is_hadamard: true,
            off_diagonal_pairs_checked: checked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HadamardCheck, Matrix};

    #[test]
    fn sylvester_four_is_hadamard() {
        let values = vec![1, 1, 1, 1, 1, -1, 1, -1, 1, 1, -1, -1, 1, -1, -1, 1];
        let matrix = Matrix::new(4, 4, values).expect("matrix");
        assert!(matrix.is_hadamard());
        assert_eq!(
            matrix.check_hadamard(),
            HadamardCheck {
                is_hadamard: true,
                off_diagonal_pairs_checked: 6,
            }
        );
    }

    #[test]
    fn checked_hadamard_stops_at_first_nonorthogonal_pair() {
        let matrix = Matrix::new(2, 2, vec![1, 1, 1, 1]).expect("matrix");
        assert_eq!(
            matrix.check_hadamard(),
            HadamardCheck {
                is_hadamard: false,
                off_diagonal_pairs_checked: 1,
            }
        );
    }

    #[test]
    fn empty_matrix_is_not_hadamard() {
        let matrix = Matrix::new(0, 0, Vec::new()).expect("matrix");
        assert_eq!(
            matrix.check_hadamard(),
            HadamardCheck {
                is_hadamard: false,
                off_diagonal_pairs_checked: 0,
            }
        );
    }
}
