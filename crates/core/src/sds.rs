use crate::exact_row_sum_square_candidates_167;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CyclicDifferenceBlock {
    order: usize,
    elements: Vec<usize>,
}

impl CyclicDifferenceBlock {
    pub fn new(order: usize, mut elements: Vec<usize>) -> Result<Self, String> {
        if order == 0 {
            return Err("cyclic difference block order must be positive".to_string());
        }
        if elements.iter().any(|value| *value >= order) {
            return Err("block element falls outside the cyclic group".to_string());
        }
        elements.sort_unstable();
        elements.dedup();
        Ok(Self { order, elements })
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn elements(&self) -> &[usize] {
        &self.elements
    }

    pub fn difference_profile(&self) -> Vec<usize> {
        let mut profile = vec![0_usize; self.order];
        for &left in &self.elements {
            for &right in &self.elements {
                if left == right {
                    continue;
                }
                let shift = (left + self.order - right) % self.order;
                profile[shift] += 1;
            }
        }
        profile
    }

    pub fn to_line(&self) -> String {
        self.elements
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupplementaryDifferenceSet {
    order: usize,
    blocks: Vec<CyclicDifferenceBlock>,
}

impl SupplementaryDifferenceSet {
    pub fn new(order: usize, blocks: Vec<CyclicDifferenceBlock>) -> Result<Self, String> {
        if order == 0 {
            return Err("supplementary difference set order must be positive".to_string());
        }
        if blocks.iter().any(|block| block.order() != order) {
            return Err("all SDS blocks must live in the same cyclic group".to_string());
        }
        Ok(Self { order, blocks })
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn blocks(&self) -> &[CyclicDifferenceBlock] {
        &self.blocks
    }

    pub fn block_sizes(&self) -> Vec<usize> {
        self.blocks.iter().map(CyclicDifferenceBlock::len).collect()
    }

    pub fn combined_difference_profile(&self) -> Vec<usize> {
        let mut combined = vec![0_usize; self.order];
        for block in &self.blocks {
            let profile = block.difference_profile();
            for (index, value) in profile.into_iter().enumerate() {
                combined[index] += value;
            }
        }
        combined
    }

    pub fn lambda(&self) -> Option<usize> {
        let combined = self.combined_difference_profile();
        let lambda = *combined.get(1)?;
        if combined.iter().skip(1).all(|value| *value == lambda) {
            Some(lambda)
        } else {
            None
        }
    }

    pub fn is_supplementary_difference_set(&self) -> bool {
        self.lambda().is_some()
    }
}

pub fn sds_target_lambda(order: usize, block_sizes: &[usize]) -> Option<usize> {
    if order <= 1 {
        return None;
    }
    let numerator = block_sizes
        .iter()
        .map(|size| size.saturating_mul(size.saturating_sub(1)))
        .sum::<usize>();
    if numerator % (order - 1) == 0 {
        Some(numerator / (order - 1))
    } else {
        None
    }
}

pub fn validate_167_parameter_table() -> bool {
    exact_row_sum_square_candidates_167()
        .into_iter()
        .all(|(row_sums, block_sizes, lambda)| {
            let row_sum_square_total = row_sums.iter().map(|value| value * value).sum::<i32>();
            let block_sizes_match = row_sums
                .iter()
                .zip(block_sizes.iter())
                .all(|(row_sum, block_size)| (167 - row_sum) / 2 == *block_size);
            let lambda_match = sds_target_lambda(167, &block_sizes.map(|value| value as usize))
                .map(|value| value as i32)
                == Some(lambda);
            row_sum_square_total == 668 && block_sizes_match && lambda_match
        })
}

#[cfg(test)]
mod tests {
    use super::{
        sds_target_lambda, validate_167_parameter_table, CyclicDifferenceBlock,
        SupplementaryDifferenceSet,
    };

    #[test]
    fn singleton_block_has_zero_difference_profile() {
        let block = CyclicDifferenceBlock::new(5, vec![0]).expect("block");
        assert_eq!(block.difference_profile(), vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn two_point_block_has_expected_difference_profile() {
        let block = CyclicDifferenceBlock::new(5, vec![0, 1]).expect("block");
        assert_eq!(block.difference_profile(), vec![0, 1, 0, 0, 1]);
    }

    #[test]
    fn small_known_sds_instance_over_z5_is_detected() {
        let sds = SupplementaryDifferenceSet::new(
            5,
            vec![
                CyclicDifferenceBlock::new(5, vec![0, 1]).expect("block a"),
                CyclicDifferenceBlock::new(5, vec![0, 2]).expect("block b"),
                CyclicDifferenceBlock::new(5, vec![]).expect("block c"),
                CyclicDifferenceBlock::new(5, vec![]).expect("block d"),
            ],
        )
        .expect("sds");
        assert!(sds.is_supplementary_difference_set());
        assert_eq!(sds.lambda(), Some(1));
        assert_eq!(sds_target_lambda(5, &[2, 2, 0, 0]), Some(1));
    }

    #[test]
    fn encoded_167_parameter_table_satisfies_square_sum_and_lambda_constraints() {
        assert!(validate_167_parameter_table());
    }
}
