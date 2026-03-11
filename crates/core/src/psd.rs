use std::f64::consts::PI;

pub trait PsdBackend {
    fn name(&self) -> &'static str;
    fn compute(&self, values: &[f64]) -> Vec<f64>;
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
static AUTOCORRELATION_BACKEND: AutocorrelationPsdBackend = AutocorrelationPsdBackend;

pub fn default_psd_backend() -> &'static dyn PsdBackend {
    &DIRECT_BACKEND
}

pub fn available_psd_backends() -> Vec<&'static str> {
    vec![DIRECT_BACKEND.name(), AUTOCORRELATION_BACKEND.name()]
}

pub fn get_psd_backend(name: &str) -> Option<&'static dyn PsdBackend> {
    match name {
        "direct" => Some(&DIRECT_BACKEND),
        "autocorrelation" => Some(&AUTOCORRELATION_BACKEND),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        available_psd_backends, default_psd_backend, get_psd_backend, AutocorrelationPsdBackend,
        DirectPsdBackend, PsdBackend,
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
        assert!(names.contains(&"autocorrelation"));
        assert_eq!(default_psd_backend().name(), "direct");
        assert!(get_psd_backend("direct").is_some());
    }

    #[test]
    fn direct_and_autocorrelation_backends_agree_on_small_input() {
        let values = [1.0, -1.0, -1.0, 1.0, 1.0];
        let direct = DirectPsdBackend.compute(&values);
        let autocorrelation = AutocorrelationPsdBackend.compute(&values);
        assert_close(&direct, &autocorrelation);
    }
}
