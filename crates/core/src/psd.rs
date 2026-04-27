use std::f64::consts::PI;

pub trait PsdBackend {
    fn name(&self) -> &'static str;
    fn compute(&self, values: &[f64]) -> Vec<f64>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Complex64 {
    real: f64,
    imag: f64,
}

impl Complex64 {
    fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    fn norm_sqr(self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }
}

impl std::ops::Add for Complex64 {
    type Output = Complex64;

    fn add(self, rhs: Self) -> Self::Output {
        Complex64::new(self.real + rhs.real, self.imag + rhs.imag)
    }
}

impl std::ops::AddAssign for Complex64 {
    fn add_assign(&mut self, rhs: Self) {
        self.real += rhs.real;
        self.imag += rhs.imag;
    }
}

impl std::ops::Mul for Complex64 {
    type Output = Complex64;

    fn mul(self, rhs: Self) -> Self::Output {
        Complex64::new(
            self.real * rhs.real - self.imag * rhs.imag,
            self.real * rhs.imag + self.imag * rhs.real,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DirectPsdBackend;

impl PsdBackend for DirectPsdBackend {
    fn name(&self) -> &'static str {
        "direct"
    }

    fn compute(&self, values: &[f64]) -> Vec<f64> {
        let n = values.len();
        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            let mut real = 0.0_f64;
            let mut imag = 0.0_f64;
            for (j, value) in values.iter().enumerate() {
                let angle = -2.0 * PI * (j * k) as f64 / n as f64;
                real += value * angle.cos();
                imag += value * angle.sin();
            }
            out.push(real * real + imag * imag);
        }
        out
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FftPsdBackend;

impl PsdBackend for FftPsdBackend {
    fn name(&self) -> &'static str {
        "fft"
    }

    fn compute(&self, values: &[f64]) -> Vec<f64> {
        fft_real(values)
            .into_iter()
            .map(Complex64::norm_sqr)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AutocorrelationPsdBackend;

impl PsdBackend for AutocorrelationPsdBackend {
    fn name(&self) -> &'static str {
        "autocorrelation"
    }

    fn compute(&self, values: &[f64]) -> Vec<f64> {
        let n = values.len();
        let mut autocorrelation = vec![0.0_f64; n];
        for shift in 0..n {
            let mut total = 0.0_f64;
            for index in 0..n {
                total += values[index] * values[(index + shift) % n];
            }
            autocorrelation[shift] = total;
        }

        let mut out = Vec::with_capacity(n);
        for k in 0..n {
            let mut real = 0.0_f64;
            let mut imag = 0.0_f64;
            for (shift, value) in autocorrelation.iter().enumerate() {
                let angle = -2.0 * PI * (shift * k) as f64 / n as f64;
                real += value * angle.cos();
                imag += value * angle.sin();
            }
            let bin = real + imag.abs();
            out.push(if bin < 0.0 { 0.0 } else { bin });
        }
        out
    }
}

static DIRECT_BACKEND: DirectPsdBackend = DirectPsdBackend;
static FFT_BACKEND: FftPsdBackend = FftPsdBackend;
static AUTOCORRELATION_BACKEND: AutocorrelationPsdBackend = AutocorrelationPsdBackend;

pub fn default_psd_backend() -> &'static dyn PsdBackend {
    &FFT_BACKEND
}

pub fn available_psd_backends() -> Vec<&'static str> {
    vec![
        DIRECT_BACKEND.name(),
        FFT_BACKEND.name(),
        AUTOCORRELATION_BACKEND.name(),
    ]
}

pub fn get_psd_backend(name: &str) -> Option<&'static dyn PsdBackend> {
    match name {
        "direct" => Some(&DIRECT_BACKEND),
        "fft" => Some(&FFT_BACKEND),
        "autocorrelation" => Some(&AUTOCORRELATION_BACKEND),
        _ => None,
    }
}

fn fft_real(values: &[f64]) -> Vec<Complex64> {
    let input = values
        .iter()
        .map(|value| Complex64::new(*value, 0.0))
        .collect::<Vec<_>>();
    fft_complex(&input)
}

fn fft_complex(values: &[Complex64]) -> Vec<Complex64> {
    let n = values.len();
    if n <= 1 {
        return values.to_vec();
    }

    let radix = smallest_factor(n);
    if radix == n {
        return direct_dft_complex(values);
    }

    let inner_len = n / radix;
    let mut inner_transforms = Vec::with_capacity(radix);
    for offset in 0..radix {
        let subsequence = (0..inner_len)
            .map(|index| values[offset + index * radix])
            .collect::<Vec<_>>();
        inner_transforms.push(fft_complex(&subsequence));
    }

    let mut out = vec![Complex64::default(); n];
    for k1 in 0..inner_len {
        for k0 in 0..radix {
            let mut total = Complex64::default();
            for offset in 0..radix {
                let twiddle = cis(-2.0 * PI * (offset * (k1 + inner_len * k0)) as f64 / n as f64);
                total += inner_transforms[offset][k1] * twiddle;
            }
            out[k1 + inner_len * k0] = total;
        }
    }
    out
}

fn direct_dft_complex(values: &[Complex64]) -> Vec<Complex64> {
    let n = values.len();
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let mut total = Complex64::default();
        for (j, value) in values.iter().enumerate() {
            total += *value * cis(-2.0 * PI * (j * k) as f64 / n as f64);
        }
        out.push(total);
    }
    out
}

fn cis(angle: f64) -> Complex64 {
    Complex64::new(angle.cos(), angle.sin())
}

fn smallest_factor(n: usize) -> usize {
    if n % 2 == 0 {
        return 2;
    }

    let mut factor = 3;
    while factor * factor <= n {
        if n % factor == 0 {
            return factor;
        }
        factor += 2;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::{
        available_psd_backends, default_psd_backend, get_psd_backend, AutocorrelationPsdBackend,
        DirectPsdBackend, FftPsdBackend, PsdBackend,
    };

    fn assert_close(left: &[f64], right: &[f64]) {
        assert_eq!(left.len(), right.len());
        for (lhs, rhs) in left.iter().zip(right.iter()) {
            assert!(
                (lhs - rhs).abs() < 1.0e-6,
                "PSD mismatch: lhs={lhs}, rhs={rhs}"
            );
        }
    }

    #[test]
    fn backends_are_registered() {
        let names = available_psd_backends();
        assert!(names.contains(&"direct"));
        assert!(names.contains(&"fft"));
        assert!(names.contains(&"autocorrelation"));
        assert_eq!(default_psd_backend().name(), "fft");
        assert!(get_psd_backend("direct").is_some());
        assert!(get_psd_backend("fft").is_some());
    }

    #[test]
    fn direct_and_autocorrelation_backends_agree_on_small_input() {
        let values = [1.0, -1.0, -1.0, 1.0, 1.0];
        let direct = DirectPsdBackend.compute(&values);
        let autocorrelation = AutocorrelationPsdBackend.compute(&values);
        assert_close(&direct, &autocorrelation);
    }

    #[test]
    fn direct_and_fft_backends_agree_on_small_input() {
        let values = [1.0, -1.0, -1.0, 1.0, 1.0];
        let direct = DirectPsdBackend.compute(&values);
        let fft = FftPsdBackend.compute(&values);
        assert_close(&direct, &fft);
    }

    #[test]
    fn fft_backend_matches_reference_across_small_normalized_sequences() {
        for length in [3_usize, 5, 7, 9, 11, 13, 15] {
            let free_bits = length - 1;
            let sequence_count = 1_u64 << free_bits;
            for index in 0..sequence_count {
                let mut values = vec![1.0_f64];
                for offset in 1..length {
                    let bit = (index >> (offset - 1)) & 1;
                    values.push(if bit == 1 { 1.0 } else { -1.0 });
                }
                let direct = DirectPsdBackend.compute(&values);
                let fft = FftPsdBackend.compute(&values);
                let autocorrelation = AutocorrelationPsdBackend.compute(&values);
                assert_close(&direct, &fft);
                assert_close(&direct, &autocorrelation);
            }
        }
    }
}
