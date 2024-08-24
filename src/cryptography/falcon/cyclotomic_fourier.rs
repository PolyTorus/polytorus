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
}