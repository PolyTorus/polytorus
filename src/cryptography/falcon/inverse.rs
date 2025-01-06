use num::{One, Zero};
use num_complex::Complex64;
use std::ops::{Mul, MulAssign};

pub trait Inverse: Copy + Zero + MulAssign + One + Mul<Output = Self> {
    fn inverse_or_zero(self) -> Self;

    fn batch_inverse_or_zero(batch: &[Self]) -> Vec<Self> {
        batch.iter().map(|&x| x.inverse_or_zero()).collect()
    }
}

impl Inverse for Complex64 {
    fn inverse_or_zero(self) -> Self {
        if self.is_zero() {
            Complex64::zero()
        } else {
            let norm_sq = self.norm_sqr();
            Complex64::new(self.re / norm_sq, -self.im / norm_sq)
        }
    }
}

impl Inverse for f64 {
    fn inverse_or_zero(self) -> Self {
        if self.is_zero() {
            0.0
        } else {
            1.0 / self
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{thread_rng, RngCore};

    #[test]
    fn test_complex_inverse() {
        let mut rng = thread_rng();
        let c = Complex64::new(rng.next_u32() as f64, rng.next_u32() as f64);
        let i = c.inverse_or_zero();
        let diff = c * i - Complex64::one();
        let norm = diff.re * diff.re + diff.im * diff.im;
        assert!(norm < f64::EPSILON * 100.0, "norm: {norm}");
    }
}
