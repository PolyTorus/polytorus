use std::{
    f64::consts::PI,
    ops::{Add, Mul, MulAssign, Sub},
};

use num::{One, Zero};
use num_complex::Complex64;

use super::inverse::Inverse;

pub trait CyclotomicFourier
where
    Self: Sized
        + Copy
        + One
        + Zero
        + Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + MulAssign
        + Inverse,
{
    fn primitive_root_of_unity(n: usize) -> Self;

    fn bitreverse_index(arg: usize, n: usize) -> usize {
        assert!(n > 0);
        assert_eq!(n & (n - 1), 0);
        let mut rev = 0;
        let mut m = n >> 1;
        let mut k = 1;
        while m > 0 {
            rev |= (((arg & m) != 0) as usize) * k;
            k <<= 1;
            m >>= 1;
        }
        rev
    }

    fn bitreversed_powers(n: usize) -> Vec<Self> {
        let psi = Self::primitive_root_of_unity(2 * n);
        let mut array = vec![Self::zero(); n];
        let mut alpha = Self::one();
        for a in array.iter_mut() {
            *a = alpha;
            alpha *= psi;
        }
        Self::bitreverse_array(&mut array);
        array
    }

    fn bitreversed_powers_inverse(n: usize) -> Vec<Self> {
        let psi = Self::primitive_root_of_unity(2 * n).inverse_or_zero();
        let mut array = vec![Self::zero(); n];
        let mut alpha = Self::one();
        for a in array.iter_mut() {
            *a = alpha;
            alpha *= psi;
        }
        Self::bitreverse_array(&mut array);
        array
    }

    fn bitreverse_array<T>(array: &mut [T]) {
        let n = array.len();
        for i in 0..n {
            let j = Self::bitreverse_index(i, n);
            if i < j {
                array.swap(i, j);
            }
        }
    }

    fn fft(a: &mut [Self], psi_rev: &[Self]) {
        let n = a.len();
        let mut t = n;
        let mut m = 1;
        while m < n {
            t >>= 1;
            for i in 0..m {
                let j1 = 2 * i * t;
                let j2 = j1 + t - 1;
                let s = psi_rev[m + i];
                for j in j1..=j2 {
                    let u = a[j];
                    let v = a[j + t] * s;
                    a[j] = u + v;
                    a[j + t] = u - v;
                }
            }
            m <<= 1;
        }
    }

    fn ifft(a: &mut [Self], psi_inv_rev: &[Self], ninv: Self) {
        let n = a.len();
        let mut t = 1;
        let mut m = n;
        while m > 1 {
            let h = m / 2;
            let mut j1 = 0;
            for i in 0..h {
                let j2 = j1 + t - 1;
                let s = psi_inv_rev[h + i];
                for j in j1..=j2 {
                    let u = a[j];
                    let v = a[j + t];
                    a[j] = u + v;
                    a[j + t] = (u - v) * s;
                }
                j1 += 2 * t;
            }
            t <<= 1;
            m >>= 1;
        }
        for ai in a.iter_mut() {
            *ai *= ninv;
        }
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
        let n = 2 * n_over_2;
        let mut f = vec![Self::zero(); n];
        for i in 0..n_over_2 {
            let two_i = i * 2;
            f[two_i] = f0[i] + psi_rev[n_over_2 + i] * f1[i];
            f[two_i + 1] = f0[i] - psi_rev[n_over_2 + i] * f1[i];
        }
        f
    }
}

impl CyclotomicFourier for Complex64 {
    fn primitive_root_of_unity(n: usize) -> Self {
        let angle = 2. * PI / (n as f64);
        Complex64::new(f64::cos(angle), f64::sin(angle))
    }

    fn bitreversed_powers(n: usize) -> Vec<Self> {
        let mut array = vec![Self::zero(); n];
        let half_circle = PI;
        for (i, a) in array.iter_mut().enumerate() {
            let angle = (i as f64) * half_circle / (n as f64);
            *a = Self::new(f64::cos(angle), f64::sin(angle));
        }
        Self::bitreverse_array(&mut array);
        array
    }

    fn bitreversed_powers_inverse(n: usize) -> Vec<Self> {
        let mut array = vec![Self::zero(); n];
        let half_circle = PI;
        for (i, a) in array.iter_mut().enumerate() {
            let angle = (i as f64) * half_circle / (n as f64);
            *a = Self::new(f64::cos(angle), -f64::sin(angle));
        }
        Self::bitreverse_array(&mut array);
        array
    }
}
