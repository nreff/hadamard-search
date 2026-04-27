use hadamard_core::{LegendrePair, Matrix, Sequence};

pub fn circulant(sequence: &Sequence) -> Matrix {
    let n = sequence.len();
    let mut values = Vec::with_capacity(n * n);
    for row in 0..n {
        for col in 0..n {
            let index = (col + n - row) % n;
            values.push(sequence.values()[index]);
        }
    }
    Matrix::new(n, n, values).expect("circulant matrix dimensions")
}

pub fn build_two_circulant_hadamard(pair: &LegendrePair) -> Result<Matrix, String> {
    if !pair.is_legendre_pair() {
        return Err("pair does not satisfy Legendre autocorrelation conditions".to_string());
    }
    if !pair.has_two_circulant_row_sums() {
        return Err(
            "pair does not satisfy row-sum=1 conditions for the 2cc construction".to_string(),
        );
    }

    let a = circulant(&pair.a);
    let b = circulant(&pair.b);
    let n = pair.a.len();
    let size = 2 * n + 2;
    let mut values = vec![0_i8; size * size];

    for col in 0..2 {
        values[col] = -1;
    }
    for col in 2..(n + 2) {
        values[col] = 1;
    }
    for col in (n + 2)..size {
        values[col] = 1;
    }

    values[size] = -1;
    values[size + 1] = 1;
    for col in 2..(n + 2) {
        values[size + col] = 1;
    }
    for col in (n + 2)..size {
        values[size + col] = -1;
    }

    for row in 0..n {
        let top_row = row + 2;
        values[top_row * size] = 1;
        values[top_row * size + 1] = 1;
        for col in 0..n {
            values[top_row * size + (col + 2)] = a.get(row, col);
            values[top_row * size + (col + n + 2)] = b.get(row, col);
        }
    }

    for row in 0..n {
        let bottom_row = row + n + 2;
        values[bottom_row * size] = 1;
        values[bottom_row * size + 1] = -1;
        for col in 0..n {
            values[bottom_row * size + (col + 2)] = b.get(col, row);
            values[bottom_row * size + (col + n + 2)] = -a.get(col, row);
        }
    }

    Matrix::new(size, size, values)
}

#[cfg(test)]
mod tests {
    use super::build_two_circulant_hadamard;
    use hadamard_core::{LegendrePair, Sequence};

    #[test]
    fn known_length_five_pair_builds_order_twelve_hadamard() {
        let a = Sequence::new(vec![1, -1, -1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, 1, -1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        let matrix = build_two_circulant_hadamard(&pair).expect("matrix");
        assert_eq!(matrix.rows(), 12);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn known_length_seven_pair_builds_order_sixteen_hadamard() {
        let a = Sequence::new(vec![1, -1, -1, 1, -1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, -1, 1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        let matrix = build_two_circulant_hadamard(&pair).expect("matrix");
        assert_eq!(matrix.rows(), 16);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn known_length_nine_pair_builds_order_twenty_hadamard() {
        let a = Sequence::new(vec![1, -1, -1, -1, 1, -1, 1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, 1, -1, 1, -1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        let matrix = build_two_circulant_hadamard(&pair).expect("matrix");
        assert_eq!(matrix.rows(), 20);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn known_length_eleven_pair_builds_order_twenty_four_hadamard() {
        let a = Sequence::new(vec![1, 1, 1, -1, 1, 1, -1, 1, -1, -1, -1]).expect("a");
        let b = Sequence::new(vec![1, 1, 1, -1, 1, 1, -1, 1, -1, -1, -1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        let matrix = build_two_circulant_hadamard(&pair).expect("matrix");
        assert_eq!(matrix.rows(), 24);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn known_length_thirteen_pair_builds_order_twenty_eight_hadamard() {
        let a = Sequence::new(vec![1, 1, 1, -1, 1, 1, 1, -1, 1, -1, -1, -1, -1]).expect("a");
        let b = Sequence::new(vec![1, -1, 1, 1, 1, -1, -1, 1, 1, -1, 1, -1, -1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        let matrix = build_two_circulant_hadamard(&pair).expect("matrix");
        assert_eq!(matrix.rows(), 28);
        assert!(matrix.is_hadamard());
    }
}
