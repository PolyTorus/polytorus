use std::f64::consts::PI;
use std::ops::{Add, Mul, MulAssign, Sub};
use num::{One, Zero};
use num_complex::Complex64;

use super::inverse::Inverse;

pub trait CyclotomicFourier
where
    Self: Copy + Sized + Zero + One + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + MulAssign + Inverse,
{
    fn primitive_root_of_unity(n: usize) -> Self;

    fn bitreverse(arg: usize, n: usize) -> usize {
        assert!(n > 0);
        assert_eq!(n & (n - 1), 0);
        
        (0..n.trailing_zeros()).fold(0, |rev, i| {
            rev | (((arg & (1 << i)) != 0) as usize) << (n.trailing_zeros() - 1 - i)
        })
    }

    fn bitreverse_array<T> (array: &mut [T]) {
        let n = array.len();
        let mut rev = vec![0; n];
        for i in 0..n {
            rev[i] = Self::bitreverse(i, n);
        }

        for i in 0..n {
            if i < rev[i] {
                array.swap(i, rev[i]);
            }
        }
    }

    fn generate_powers(n: usize, inverse: bool) -> Vec<Self> {
        let psi = if inverse {
            Self::primitive_root_of_unity(2 * n).inverse_or_zero()
        } else {
            Self::primitive_root_of_unity(2 * n)
        };

        let mut array = vec![Self::one(); n];

        for i in 1..n {
            array[i] = array[i - 1] * psi;
        }

        Self::bitreverse_array(&mut array);
        array
    }

    fn fft_general(a: &mut [Self], psi_powers: &[Self], inverse: bool) {
        let n = a.len();
        let mut t = if inverse { 1 } else { n };
        let mut m = if inverse { n } else { 1 };

        while if inverse { m > 1} else { m < n } {
            if inverse {
                let h = m / 2;
                let mut j1 = 0;
                
                for _ in 0..h {
                    let j2 = j1 + t - 1;
                    let s = psi_powers[h + 1];

                    for j in j1..=j2 {
                        let u = a[j];
                        let v = a[j + h] * s;
                        a[j] = u + v;
                        a[j + h] = u - v;
                    }

                    j1 = j2 + t + 1;
                }

                t *= 2;
                m /= 2;
            } else {
                let h = m * 2;
                let mut j1 = 0;

                for _ in 0..m {
                    let j2 = j1 + t - 1;
                    let s = psi_powers[h + 1];

                    for j in j1..=j2 {
                        let u = a[j];
                        let v = a[j + t];
                        a[j] = u + v;
                        a[j + t] = (u - v) * s;
                    }

                    j1 = j2 + t + 1;
                }

                t /= 2;
                m *= 2;
            }
        }
    }

    fn fft(a: &mut [Self], psi_rev: &[Self]) {
        Self::fft_general(a, psi_rev, false);
    }

    fn ifft(a: &mut [Self], psi_inv_rev: &[Self]) {
        Self::fft_general(a, psi_inv_rev, false);
    }

    fn split_fft(f: &[Self], psi_inv_rev: &[Self]) -> (Vec<Self>, Vec<Self>) {
        let n_over_2 = f.len() / 2;
        let mut f0 = vec![Self::zero(); n_over_2];
        let mut f1 = vec![Self::zero(); n_over_2];

        let two_inv = (Self::one() + Self::one()).inverse_or_zero();
        for i in 0..n_over_2 {
            let two_i = i * 2;
            let two_zeta_inv = two_inv * psi_inv_rev[n_over_2 + i];
            f0[i] = two_inv * (f[two_i] + f[two_i + 1]);
            f1[i] = two_zeta_inv * (f[two_i] - f[two_i + 1]);
        }
        (f0, f1)
    }

    fn merge_fft(f0: &[Self], f1: &[Self], psi_rev: &[Self]) -> Vec<Self> {
        let n_over_2 = f0.len();
        let mut f = vec![Self::zero(); n_over_2 * 2];
        for i in 0..n_over_2 {
            let two_i = i * 2;
            f[two_i] = f0[i] + psi_rev[n_over_2 + i] * f1[i];
            f[two_i + 1] = f0[i] - psi_rev[n_over_2 + i] * f1[i];
        }
        f
    }

    fn from_usize(n: usize) -> Self;

}

impl CyclotomicFourier for Complex64 {
    fn primitive_root_of_unity(n: usize) -> Self {
        let angle = 2. * PI / (n as f64);
        Complex64::new(f64::cos(angle), f64::sin(angle))
    }

    fn generate_powers(n: usize, inverse: bool) -> Vec<Self> {
        let mut array = vec![Complex64::zero(); n];
        let half_circle = PI;

        for (i, a) in array.iter_mut().enumerate() {
            let angle = (i as f64) * half_circle / (n as f64);
            *a = if inverse {
                Complex64::new(f64::cos(angle), -f64::sin(angle))
            } else {
                Complex64::new(f64::cos(angle), f64::sin(angle))
            };
        }

        Self::bitreverse_array(&mut array);
        array
    }

    fn from_usize(n: usize) -> Self {
        Complex64::new(n as f64, 0.0)
    }
}