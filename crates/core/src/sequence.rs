use crate::psd::{default_psd_backend, PsdBackend};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence {
    values: Vec<i8>,
}

impl Sequence {
    pub fn new(values: Vec<i8>) -> Result<Self, String> {
        if values.is_empty() {
            return Err("sequence must be non-empty".to_string());
        }
        if values.iter().any(|value| *value != -1 && *value != 1) {
            return Err("sequence values must be +/-1".to_string());
        }
        Ok(Self { values })
    }

    pub fn from_bits(length: usize, bits: u64) -> Result<Self, String> {
        if length == 0 {
            return Err("sequence length must be positive".to_string());
        }
        if length > 64 {
            return Err("from_bits supports length <= 64".to_string());
        }
        let mut values = Vec::with_capacity(length);
        for index in 0..length {
            let is_one = ((bits >> index) & 1) == 1;
            values.push(if is_one { 1 } else { -1 });
        }
        Self::new(values)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &[i8] {
        &self.values
    }

    pub fn is_normalized(&self) -> bool {
        self.values.first() == Some(&1)
    }

    pub fn row_sum(&self) -> i32 {
        self.values.iter().map(|value| i32::from(*value)).sum()
    }

    pub fn periodic_autocorrelation(&self, shift: usize) -> i32 {
        let n = self.values.len();
        let offset = shift % n;
        let mut total = 0_i32;
        for index in 0..n {
            total +=
                i32::from(self.values[index]) * i32::from(self.values[(index + offset) % n]);
        }
        total
    }

    pub fn psd(&self) -> Vec<f64> {
        self.psd_with_backend(default_psd_backend())
    }

    pub fn psd_with_backend(&self, backend: &dyn PsdBackend) -> Vec<f64> {
        let values = self
            .values
            .iter()
            .map(|value| f64::from(*value))
            .collect::<Vec<_>>();
        backend.compute(&values)
    }

    pub fn compress(&self, factor: usize) -> Result<CompressedSequence, String> {
        if factor == 0 {
            return Err("compression factor must be positive".to_string());
        }
        if self.len() % factor != 0 {
            return Err("length must be divisible by compression factor".to_string());
        }
        let reduced = self.len() / factor;
        let mut values = vec![0_i16; reduced];
        for index in 0..reduced {
            let mut total = 0_i16;
            for layer in 0..factor {
                total += i16::from(self.values[index + layer * reduced]);
            }
            values[index] = total;
        }
        Ok(CompressedSequence { factor, values })
    }

    pub fn to_line(&self) -> String {
        self.values
            .iter()
            .map(|value| if *value == 1 { '+' } else { '-' })
            .collect()
    }

    pub fn rotate(&self, shift: usize) -> Sequence {
        let n = self.values.len();
        let mut values = Vec::with_capacity(n);
        for index in 0..n {
            values.push(self.values[(index + shift) % n]);
        }
        Sequence { values }
    }

    pub fn canonical_normalized_rotation_line(&self) -> Option<String> {
        let mut best: Option<String> = None;
        for shift in 0..self.len() {
            let rotated = self.rotate(shift);
            if !rotated.is_normalized() {
                continue;
            }
            let line = rotated.to_line();
            if best.as_ref().map_or(true, |current| line < *current) {
                best = Some(line);
            }
        }
        best
    }

    pub fn is_canonical_normalized_rotation(&self) -> bool {
        if !self.is_normalized() {
            return false;
        }
        self.canonical_normalized_rotation_line()
            .map_or(false, |line| line == self.to_line())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompressedSequence {
    factor: usize,
    values: Vec<i16>,
}

impl CompressedSequence {
    pub fn new(factor: usize, values: Vec<i16>) -> Result<Self, String> {
        if factor == 0 {
            return Err("compression factor must be positive".to_string());
        }
        let alphabet = Self::alphabet_for_factor(factor);
        if values.iter().any(|value| !alphabet.contains(value)) {
            return Err("compressed value falls outside valid alphabet".to_string());
        }
        Ok(Self { factor, values })
    }

    pub fn factor(&self) -> usize {
        self.factor
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> &[i16] {
        &self.values
    }

    pub fn row_sum(&self) -> i32 {
        self.values.iter().map(|value| i32::from(*value)).sum()
    }

    pub fn squared_norm(&self) -> i32 {
        self.values
            .iter()
            .map(|value| i32::from(*value) * i32::from(*value))
            .sum()
    }

    pub fn periodic_autocorrelation(&self, shift: usize) -> i32 {
        let n = self.values.len();
        let offset = shift % n;
        let mut total = 0_i32;
        for index in 0..n {
            total += i32::from(self.values[index]) * i32::from(self.values[(index + offset) % n]);
        }
        total
    }

    pub fn compressed_legendre_residual_against(&self, other: &Self) -> i64 {
        let n = self.len();
        let target = -2_i64 * self.factor as i64;
        let mut total = 0_i64;
        for shift in 1..n {
            let lhs = i64::from(self.periodic_autocorrelation(shift));
            let rhs = i64::from(other.periodic_autocorrelation(shift));
            total += (lhs + rhs - target).abs();
        }
        total
    }

    pub fn psd_with_backend(&self, backend: &dyn PsdBackend) -> Vec<f64> {
        let values = self
            .values
            .iter()
            .map(|value| f64::from(*value))
            .collect::<Vec<_>>();
        backend.compute(&values)
    }

    pub fn compressed_psd_residual_against(
        &self,
        other: &Self,
        backend: &dyn PsdBackend,
    ) -> f64 {
        let psd_a = self.psd_with_backend(backend);
        let psd_b = other.psd_with_backend(backend);
        let target = f64::from(self.squared_norm() + other.squared_norm()) + 2.0 * self.factor as f64;
        let mut total = 0.0_f64;
        for index in 1..self.len() {
            total += ((psd_a[index] + psd_b[index]) - target).abs();
        }
        total
    }

    pub fn alphabet_for_factor(factor: usize) -> Vec<i16> {
        let mut values = Vec::new();
        let mut current = -(factor as i16);
        while current <= factor as i16 {
            values.push(current);
            current += 2;
        }
        values
    }

    pub fn to_line(&self) -> String {
        self.values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegendrePair {
    pub a: Sequence,
    pub b: Sequence,
}

impl LegendrePair {
    pub fn new(a: Sequence, b: Sequence) -> Result<Self, String> {
        if a.len() != b.len() {
            return Err("Legendre pair sequences must have the same length".to_string());
        }
        if a.len() % 2 == 0 {
            return Err("Legendre pair length must be odd".to_string());
        }
        Ok(Self { a, b })
    }

    pub fn is_legendre_pair(&self) -> bool {
        for shift in 1..self.a.len() {
            if self.a.periodic_autocorrelation(shift) + self.b.periodic_autocorrelation(shift) != -2 {
                return false;
            }
        }
        true
    }

    pub fn has_two_circulant_row_sums(&self) -> bool {
        self.a.row_sum() == 1 && self.b.row_sum() == 1
    }

    pub fn canonical_common_shift_pair(&self) -> Option<(Sequence, Sequence)> {
        let mut best: Option<(String, Sequence, Sequence)> = None;
        for shift in 0..self.a.len() {
            let a_rot = self.a.rotate(shift);
            let b_rot = self.b.rotate(shift);
            if !a_rot.is_normalized() || !b_rot.is_normalized() {
                continue;
            }

            let a_line = a_rot.to_line();
            let b_line = b_rot.to_line();
            let (left_line, right_line, left_seq, right_seq) = if a_line <= b_line {
                (a_line, b_line, a_rot, b_rot)
            } else {
                (b_line, a_line, b_rot, a_rot)
            };
            let key = format!("{left_line}|{right_line}");
            if best.as_ref().map_or(true, |current| key < current.0) {
                best = Some((key, left_seq, right_seq));
            }
        }
        best.map(|(_, left, right)| (left, right))
    }
}

pub fn exact_row_sum_square_candidates_167() -> Vec<([i32; 4], [i32; 4], i32)> {
    vec![
        ([25, 5, 3, 3], [71, 81, 82, 82], 149),
        ([23, 11, 3, 3], [72, 78, 82, 82], 147),
        ([23, 9, 7, 3], [72, 79, 80, 82], 146),
        ([21, 15, 1, 1], [73, 76, 83, 83], 148),
        ([21, 13, 7, 3], [73, 77, 80, 82], 145),
        ([21, 11, 9, 5], [73, 78, 79, 81], 144),
        ([19, 17, 3, 3], [74, 75, 82, 82], 146),
        ([19, 15, 9, 1], [74, 76, 79, 83], 145),
        ([17, 17, 9, 3], [75, 75, 79, 82], 144),
        ([15, 15, 13, 7], [76, 76, 77, 80], 142),
    ]
}

pub fn is_prime_like_target(order: usize) -> bool {
    order % 4 == 0 && matches!(order / 4, 167 | 179 | 223)
}

#[cfg(test)]
mod tests {
    use super::{CompressedSequence, LegendrePair, Sequence};
    use crate::psd::{AutocorrelationPsdBackend, DirectPsdBackend};

    #[test]
    fn sequence_compression_matches_expected_length_five_case() {
        let sequence = Sequence::new(vec![1, -1, -1, 1, 1]).expect("sequence");
        let compressed = sequence.compress(5).expect("compressed");
        assert_eq!(compressed.values(), &[1]);
    }

    #[test]
    fn compressed_sequence_alphabet_is_odd_progression() {
        assert_eq!(CompressedSequence::alphabet_for_factor(3), vec![-3, -1, 1, 3]);
    }

    #[test]
    fn known_length_five_pair_is_legendre_pair() {
        let a = Sequence::new(vec![1, -1, -1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, 1, -1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        assert!(pair.is_legendre_pair());
        assert!(pair.has_two_circulant_row_sums());
    }

    #[test]
    fn known_length_seven_pair_is_legendre_pair() {
        let a = Sequence::new(vec![1, -1, -1, 1, -1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, -1, 1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        assert!(pair.is_legendre_pair());
        assert!(pair.has_two_circulant_row_sums());
    }

    #[test]
    fn psd_backends_agree_for_sequence() {
        let sequence = Sequence::new(vec![1, -1, -1, 1, 1]).expect("sequence");
        let direct = sequence.psd_with_backend(&DirectPsdBackend);
        let autocorrelation = sequence.psd_with_backend(&AutocorrelationPsdBackend);
        assert_eq!(direct.len(), autocorrelation.len());
        for (lhs, rhs) in direct.iter().zip(autocorrelation.iter()) {
            assert!((lhs - rhs).abs() < 1.0e-6, "lhs={lhs}, rhs={rhs}");
        }
    }

    #[test]
    fn compressed_length_nine_known_case_has_zero_residual() {
        let a = Sequence::new(vec![1, -1, -1, -1, 1, -1, 1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, 1, -1, 1, -1, 1]).expect("b");
        let compressed_a = a.compress(3).expect("compressed a");
        let compressed_b = b.compress(3).expect("compressed b");
        assert_eq!(compressed_a.values(), &[1, 1, -1]);
        assert_eq!(compressed_b.values(), &[3, -1, -1]);
        assert_eq!(compressed_a.compressed_legendre_residual_against(&compressed_b), 0);
        let psd_residual =
            compressed_a.compressed_psd_residual_against(&compressed_b, &DirectPsdBackend);
        assert!(psd_residual.abs() < 1.0e-6, "psd_residual={psd_residual}");
    }

    #[test]
    fn known_length_nine_pair_is_legendre_pair() {
        let a = Sequence::new(vec![1, -1, -1, -1, 1, -1, 1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, 1, -1, 1, -1, 1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        assert!(pair.is_legendre_pair());
        assert!(pair.has_two_circulant_row_sums());
    }

    #[test]
    fn known_length_eleven_pair_is_legendre_pair() {
        let a = Sequence::new(vec![1, 1, 1, -1, 1, 1, -1, 1, -1, -1, -1]).expect("a");
        let b = Sequence::new(vec![1, 1, 1, -1, 1, 1, -1, 1, -1, -1, -1]).expect("b");
        let pair = LegendrePair::new(a, b).expect("pair");
        assert!(pair.is_legendre_pair());
        assert!(pair.has_two_circulant_row_sums());
    }

    #[test]
    fn legendre_pair_common_shift_canonicalization_is_shift_invariant() {
        let a = Sequence::new(vec![1, -1, -1, -1, 1, -1, 1, 1, 1]).expect("a");
        let b = Sequence::new(vec![1, -1, -1, 1, 1, -1, 1, -1, 1]).expect("b");
        let pair = LegendrePair::new(a.clone(), b.clone()).expect("pair");
        let shifted = LegendrePair::new(a.rotate(3), b.rotate(3)).expect("shifted");
        assert_eq!(
            pair.canonical_common_shift_pair()
                .map(|(x, y)| (x.to_line(), y.to_line())),
            shifted
                .canonical_common_shift_pair()
                .map(|(x, y)| (x.to_line(), y.to_line()))
        );
    }

    #[test]
    fn canonical_normalized_rotation_detects_known_representative() {
        let sequence = Sequence::new(vec![1, -1, -1, -1, 1, -1, 1, 1, 1]).expect("sequence");
        let rotated = sequence.rotate(3);
        let canonical = sequence
            .canonical_normalized_rotation_line()
            .expect("canonical line");
        assert_eq!(
            canonical,
            rotated
                .canonical_normalized_rotation_line()
                .expect("canonical line")
        );
        assert_ne!(sequence.to_line(), rotated.to_line());
        assert!(sequence.is_normalized());
        assert!(!rotated.is_normalized() || sequence.to_line() != canonical);
    }
}
