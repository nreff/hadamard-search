use hadamard_core::{
    CyclicDifferenceBlock, LegendrePair, Matrix, Sequence, SupplementaryDifferenceSet,
};

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

fn validate_spence_sequences(sequences: &[&Sequence; 4]) -> Result<usize, String> {
    let order = sequences[0].len();
    if sequences.iter().any(|sequence| sequence.len() != order) {
        return Err("Spence sequences must have equal lengths".to_string());
    }
    if !order.is_multiple_of(2) {
        return Err("Spence sequences must have even length".to_string());
    }

    let row_sums = sequences.map(|sequence| sequence.row_sum());
    if row_sums != [2, 0, 0, 0] {
        return Err(format!(
            "Spence sequence row sums must be [2, 0, 0, 0], got {row_sums:?}"
        ));
    }

    for shift in 1..order {
        let correlation_sum = sequences
            .iter()
            .map(|sequence| sequence.periodic_autocorrelation(shift))
            .sum::<i32>();
        if correlation_sum != -4 {
            return Err(format!(
                "Spence sequences must have combined periodic autocorrelation -4 at shift {shift}, got {correlation_sum}"
            ));
        }
    }

    Ok(order)
}

/// Build the Wallis-Whiteman bordered array from a normalized type-H4* seed.
///
/// This array is also presented in the literature as a Spence SDS construction;
/// the function name follows that common terminology without making a claim about
/// how any particular seed was discovered.
pub fn build_spence_hadamard(sequences: [&Sequence; 4]) -> Result<Matrix, String> {
    let order = validate_spence_sequences(&sequences)?;
    let size = order
        .checked_add(1)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| "Spence matrix order overflow".to_string())?;
    let value_count = size
        .checked_mul(size)
        .ok_or_else(|| "Spence matrix value count overflow".to_string())?;
    let mut values = vec![0_i8; value_count];
    let circulants = sequences.map(circulant);

    let top_left = [
        [-1, 1, 1, -1],
        [1, -1, 1, -1],
        [1, 1, -1, -1],
        [-1, -1, -1, -1],
    ];
    let top_right = [
        [-1, -1, -1, 1],
        [-1, -1, 1, -1],
        [-1, 1, -1, -1],
        [1, -1, -1, -1],
    ];
    let bottom_left = [
        [1, 1, 1, -1],
        [-1, -1, 1, -1],
        [-1, 1, -1, -1],
        [1, -1, -1, -1],
    ];

    for row in 0..4 {
        for col in 0..4 {
            values[row * size + col] = top_left[row][col];
        }
        for block_col in 0..4 {
            for col in 0..order {
                values[row * size + 4 + block_col * order + col] = top_right[row][block_col];
            }
        }
    }

    for (block_row, border) in bottom_left.iter().enumerate() {
        for row in 0..order {
            let target_row = 4 + block_row * order + row;
            for col in 0..4 {
                values[target_row * size + col] = border[col];
            }
        }
    }

    let core_blocks = [
        [
            (0, false, false, 1),
            (1, false, true, 1),
            (2, false, true, 1),
            (3, false, true, 1),
        ],
        [
            (1, false, true, -1),
            (0, false, false, 1),
            (3, true, true, 1),
            (2, true, true, -1),
        ],
        [
            (2, false, true, -1),
            (3, true, true, -1),
            (0, false, false, 1),
            (1, true, true, 1),
        ],
        [
            (3, false, true, -1),
            (2, true, true, 1),
            (1, true, true, -1),
            (0, false, false, 1),
        ],
    ];

    for (block_row, block_specs) in core_blocks.iter().enumerate() {
        for (block_col, &(source, transpose, reverse_columns, sign)) in
            block_specs.iter().enumerate()
        {
            for row in 0..order {
                for col in 0..order {
                    let source_col = if reverse_columns {
                        order - 1 - col
                    } else {
                        col
                    };
                    let entry = if transpose {
                        circulants[source].get(source_col, row)
                    } else {
                        circulants[source].get(row, source_col)
                    };
                    let target_row = 4 + block_row * order + row;
                    let target_col = 4 + block_col * order + col;
                    values[target_row * size + target_col] = sign * entry;
                }
            }
        }
    }

    Matrix::new(size, size, values)
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

pub fn sequence_from_difference_block(block: &CyclicDifferenceBlock) -> Result<Sequence, String> {
    let mut values = vec![1_i8; block.order()];
    for &index in block.elements() {
        values[index] = -1;
    }
    Sequence::new(values)
}

pub fn build_goethals_seidel(sequences: [&Sequence; 4]) -> Result<Matrix, String> {
    let order = sequences[0].len();
    if sequences.iter().any(|sequence| sequence.len() != order) {
        return Err("Goethals-Seidel sequences must have equal lengths".to_string());
    }
    for shift in 1..order {
        let correlation_sum = sequences
            .iter()
            .map(|sequence| sequence.periodic_autocorrelation(shift))
            .sum::<i32>();
        if correlation_sum != 0 {
            return Err(format!(
                "Goethals-Seidel sequences are not complementary at shift {shift}"
            ));
        }
    }

    let circulants = sequences.map(circulant);
    let size = 4 * order;
    let mut values = vec![0_i8; size * size];
    let blocks = [
        [
            (0, false, false, 1),
            (1, false, true, -1),
            (2, false, true, -1),
            (3, false, true, -1),
        ],
        [
            (1, false, true, 1),
            (0, false, false, 1),
            (3, true, true, -1),
            (2, true, true, 1),
        ],
        [
            (2, false, true, 1),
            (3, true, true, 1),
            (0, false, false, 1),
            (1, true, true, -1),
        ],
        [
            (3, false, true, 1),
            (2, true, true, -1),
            (1, true, true, 1),
            (0, false, false, 1),
        ],
    ];

    for block_row in 0..4 {
        for block_col in 0..4 {
            let (source, transpose, reverse_columns, sign) = blocks[block_row][block_col];
            for row in 0..order {
                for col in 0..order {
                    let source_col = if reverse_columns {
                        order - 1 - col
                    } else {
                        col
                    };
                    let entry = if transpose {
                        circulants[source].get(source_col, row)
                    } else {
                        circulants[source].get(row, source_col)
                    };
                    let target_row = block_row * order + row;
                    let target_col = block_col * order + col;
                    values[target_row * size + target_col] = sign * entry;
                }
            }
        }
    }

    Matrix::new(size, size, values)
}

pub fn build_goethals_seidel_from_sds(sds: &SupplementaryDifferenceSet) -> Result<Matrix, String> {
    if sds.blocks().len() != 4 {
        return Err("Goethals-Seidel construction requires exactly four SDS blocks".to_string());
    }
    if !sds.is_supplementary_difference_set() {
        return Err("blocks do not form a supplementary difference set".to_string());
    }
    let block_size_sum = sds
        .blocks()
        .iter()
        .map(CyclicDifferenceBlock::len)
        .sum::<usize>();
    let expected_lambda = block_size_sum.checked_sub(sds.order()).ok_or_else(|| {
        "SDS parameters do not satisfy the Goethals-Seidel lambda condition".to_string()
    })?;
    if sds.lambda() != Some(expected_lambda) {
        return Err("SDS parameters do not satisfy lambda = sum(k_i) - n".to_string());
    }

    let sequences = sds
        .blocks()
        .iter()
        .map(sequence_from_difference_block)
        .collect::<Result<Vec<_>, _>>()?;
    build_goethals_seidel([&sequences[0], &sequences[1], &sequences[2], &sequences[3]])
}

#[cfg(test)]
mod tests {
    use super::{
        build_goethals_seidel_from_sds, build_spence_hadamard, build_two_circulant_hadamard,
    };
    use hadamard_core::{
        CyclicDifferenceBlock, LegendrePair, Sequence, SupplementaryDifferenceSet,
    };

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

    #[test]
    fn small_sds_builds_order_twenty_goethals_seidel_hadamard() {
        let sds = SupplementaryDifferenceSet::new(
            5,
            vec![
                CyclicDifferenceBlock::new(5, vec![0]).expect("block a"),
                CyclicDifferenceBlock::new(5, vec![0]).expect("block b"),
                CyclicDifferenceBlock::new(5, vec![0, 1]).expect("block c"),
                CyclicDifferenceBlock::new(5, vec![0, 2]).expect("block d"),
            ],
        )
        .expect("sds");
        assert_eq!(sds.lambda(), Some(1));

        let matrix = build_goethals_seidel_from_sds(&sds).expect("Goethals-Seidel matrix");
        assert_eq!(matrix.rows(), 20);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn length_two_spence_sequences_build_order_twelve_hadamard() {
        let s0 = Sequence::new(vec![1, 1]).expect("s0");
        let s1 = Sequence::new(vec![1, -1]).expect("s1");
        let s2 = Sequence::new(vec![1, -1]).expect("s2");
        let s3 = Sequence::new(vec![1, -1]).expect("s3");

        let matrix = build_spence_hadamard([&s0, &s1, &s2, &s3]).expect("Spence matrix");
        assert_eq!(matrix.rows(), 12);
        assert_eq!(matrix.cols(), 12);
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn length_four_spence_sequences_exercise_nontrivial_reversal_and_transpose() {
        let s0 = Sequence::new(vec![1, 1, 1, -1]).expect("s0");
        let s1 = Sequence::new(vec![1, 1, -1, -1]).expect("s1");
        let s2 = Sequence::new(vec![1, 1, -1, -1]).expect("s2");
        let s3 = Sequence::new(vec![1, -1, 1, -1]).expect("s3");

        let matrix = build_spence_hadamard([&s0, &s1, &s2, &s3]).expect("Spence matrix");
        assert_eq!((matrix.rows(), matrix.cols()), (20, 20));
        assert!(matrix.is_hadamard());
    }

    #[test]
    fn spence_validation_rejects_bad_length_row_sum_and_correlation_inputs() {
        let plus_two = Sequence::new(vec![1, 1]).expect("plus two");
        let balanced_two = Sequence::new(vec![1, -1]).expect("balanced two");
        let balanced_four = Sequence::new(vec![1, 1, -1, -1]).expect("balanced four");
        assert_eq!(
            build_spence_hadamard([&plus_two, &balanced_four, &balanced_two, &balanced_two,])
                .expect_err("unequal lengths must fail"),
            "Spence sequences must have equal lengths"
        );

        let odd = Sequence::new(vec![1, 1, -1]).expect("odd");
        assert_eq!(
            build_spence_hadamard([&odd, &odd, &odd, &odd]).expect_err("odd lengths must fail"),
            "Spence sequences must have even length"
        );

        assert_eq!(
            build_spence_hadamard([&plus_two, &plus_two, &balanced_two, &balanced_two])
                .expect_err("wrong row sums must fail"),
            "Spence sequence row sums must be [2, 0, 0, 0], got [2, 2, 0, 0]"
        );

        let sum_two_four = Sequence::new(vec![1, 1, 1, -1]).expect("sum two four");
        assert_eq!(
            build_spence_hadamard([
                &sum_two_four,
                &balanced_four,
                &balanced_four,
                &balanced_four,
            ])
            .expect_err("wrong periodic autocorrelation must fail"),
            "Spence sequences must have combined periodic autocorrelation -4 at shift 1, got 0"
        );
    }

    #[test]
    fn known_spence_sequences_reconstruct_order_668_fixture_byte_for_byte() {
        let fixture = include_str!("../../../fixtures/known/hm-668/spence-sequences.txt");
        let sequences = fixture
            .lines()
            .map(|line| {
                assert!(line.bytes().all(|byte| byte == b'+' || byte == b'-'));
                Sequence::new(
                    line.bytes()
                        .map(|byte| if byte == b'+' { 1 } else { -1 })
                        .collect(),
                )
                .expect("valid Spence fixture sequence")
            })
            .collect::<Vec<_>>();
        assert_eq!(sequences.len(), 4);

        let matrix =
            build_spence_hadamard([&sequences[0], &sequences[1], &sequences[2], &sequences[3]])
                .expect("order-668 Spence matrix");
        assert_eq!(matrix.rows(), 668);
        assert_eq!(matrix.cols(), 668);

        let mut encoded = String::with_capacity(matrix.rows() * (matrix.cols() + 1));
        for row in 0..matrix.rows() {
            for col in 0..matrix.cols() {
                encoded.push(if matrix.get(row, col) == 1 { '+' } else { '-' });
            }
            encoded.push('\n');
        }
        assert_eq!(
            encoded.as_bytes(),
            include_bytes!("../../../fixtures/known/hm-668/matrix.txt")
        );
    }
}
