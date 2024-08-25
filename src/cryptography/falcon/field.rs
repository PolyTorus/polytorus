use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};
use rand_distr::{num_traits::{One, Zero}, Distribution, Standard};
use super::cyclotomic_fourier::CyclotomicFourier;
use super::inverse::Inverse;

pub const Q: u32 = 12 * 1024 + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Felt(u32);

impl Felt {
    #[inline]
    pub const fn new(value: i16) -> Self {
        let reduced = if value >= 0 {
            value % (Q as i16)
        } else {
            (value % (Q as i16) + Q as i16) % (Q as i16)
        };
        Felt(reduced as u32)
    }

    #[inline]
    pub const fn value(&self) -> i16 {
        self.0 as i16
    }

    #[inline]
    pub fn balanced_value(&self) -> i16 {
        let value = self.value();
        if value > (Q as i16) / 2 {
            value - Q as i16
        } else {
            value
        }
    }

    #[inline]
    pub const fn multiply(&self, other: Self) -> Self {
        Felt((self.0 as u64 * other.0 as u64 % Q as u64) as u32)
    }
}

impl From<usize> for Felt {
    #[inline]
    fn from(value: usize) -> Self {
        Felt::new((value % Q as usize) as i16)
    }
}

impl Add for Felt {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.0 as u64 + rhs.0 as u64;
        Felt((sum % Q as u64) as u32)
    }
}

impl AddAssign for Felt {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Felt {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign for Felt {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Felt {
    type Output = Felt;

    #[inline]
    fn neg(self) -> Self::Output {
        if self.0 == 0 {
            self
        } else {
            Felt(Q - self.0)
        }
    }
}

impl Mul for Felt {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl MulAssign for Felt {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Zero for Felt {
    #[inline]
    fn zero() -> Self {
        Felt(0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl One for Felt {
    #[inline]
    fn one() -> Self {
        Felt(1)
    }
}

impl Distribution<Felt> for Standard {
    #[inline]
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Felt {
        Felt((rng.next_u32() % Q) as u32)
    }
}

impl Display for Felt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl Inverse for Felt {
    fn inverse_or_zero(self) -> Self {
        if self.is_zero() {
            return Felt::zero();
        }

        let mut a = self.0 as i64;
        let mut b = Q as i64;
        let mut x = 1i64;
        let mut y = 0i64;

        while a != 0 {
            let q = b / a;
            let temp = b % a;
            b = a;
            a = temp;

            let temp = y - q * x;
            y = x;
            x = temp;
        }

        if b != 1 {
            return Felt::zero(); // No inverse exists
        }

        Felt::new((y % Q as i64) as i16)
    }
}

impl Div for Felt {
    type Output = Felt;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            panic!("Cannot divide by zero");
        } else {
            self * rhs.inverse_or_zero()
        }
    }
}

impl CyclotomicFourier for Felt {
    fn primitive_root_of_unity(n: usize) -> Self {
        let log2n = n.ilog2();
        assert!(log2n <= 12);
        // 1331 is a twelfth root of unity
        let mut a = Felt::new(1331);
        for _ in 0..(12 - log2n) {
            a *= a;
        }
        a
    }
    
    #[inline]
    fn from_usize(n: usize) -> Self {
        Felt::from(n)
    }
}