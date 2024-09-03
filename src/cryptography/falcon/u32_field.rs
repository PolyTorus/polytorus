use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

use rand_distr::{
    num_traits::{One, Zero},
    Distribution, Standard,
};

use super::cyclotomic_fourier::CyclotomicFourier;
use super::inverse::Inverse;

const Q: u32 = 1073754113u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U32Field(pub(crate) u32);

impl U32Field {
    pub const fn new(value: i32) -> Self {
        let gtz_bool = value >= 0;
        let gtz_int = gtz_bool as i32;
        let gtz_sign = gtz_int - ((!gtz_bool) as i32);
        let reduced = gtz_sign * ((gtz_sign * value) % (Q as i32));
        let canonical_representative = (reduced + (Q as i32) * (1 - gtz_int)) as u32;
        U32Field(canonical_representative)
    }

    pub const fn value(&self) -> i32 {
        self.0 as i32
    }

    pub fn balanced_value(&self) -> i32 {
        let value = self.value();
        let g = (value > ((Q as i32) / 2)) as i32;
        value - (Q as i32) * g
    }

    pub const fn multiply(&self, other: Self) -> Self {
        U32Field((((self.0 as u64) * (other.0 as u64)) % (Q as u64)) as u32)
    }

    pub fn from_usize(value: usize) -> Self {
        U32Field::new(value as i32)
    }
}

impl From<usize> for U32Field {
    fn from(value: usize) -> Self {
        U32Field::new(value as i32)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add for U32Field {
    fn add(self, rhs: Self) -> Self::Output {
        let (s, _) = self.0.overflowing_add(rhs.0);
        let (d, n) = s.overflowing_sub(Q);
        let (r, _) = d.overflowing_add(Q * (n as u32));
        U32Field(r)
    }

    type Output = Self;
}

impl AddAssign for U32Field {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for U32Field {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign for U32Field {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for U32Field {
    type Output = U32Field;

    fn neg(self) -> Self::Output {
        let is_nonzero = self.0 != 0;
        let r = Q - self.0;
        U32Field(r * (is_nonzero as u32))
    }
}

impl Mul for U32Field {
    fn mul(self, rhs: Self) -> Self::Output {
        U32Field((((self.0 as u64) * (rhs.0 as u64)) % (Q as u64)) as u32)
    }

    type Output = Self;
}

impl MulAssign for U32Field {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Zero for U32Field {
    fn zero() -> Self {
        U32Field::new(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl One for U32Field {
    fn one() -> Self {
        U32Field::new(1)
    }
}

impl Distribution<U32Field> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> U32Field {
        U32Field::new(((rng.next_u32() >> 1) % Q) as i32)
    }
}

impl Display for U32Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.value()))
    }
}

impl Inverse for U32Field {
    fn inverse_or_zero(self) -> Self {
        let q_minus_two = Q - 2;
        let mut acc = U32Field(1);
        let mut mask = u32::MAX - (u32::MAX >> 1);
        for _ in 0..32 {
            acc = acc * acc;
            if mask & q_minus_two != 0 {
                acc = acc * self;
            }
            mask = mask >> 1;
        }
        acc
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Div for U32Field {
    type Output = U32Field;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            panic!("Cannot divide by zero");
        } else {
            self.multiply(rhs.inverse_or_zero())
        }
    }
}


impl CyclotomicFourier for U32Field {
    fn primitive_root_of_unity(n: usize) -> Self {
        let log2n = n.ilog2();
        assert!(log2n <= 12);
        
        let mut a = U32Field::new(48440i32);
        let num_squarings = 12 - n.ilog2();
        for _ in 0..num_squarings {
            a *= a;
        }
        a
    }
    
    fn from_usize(n: usize) -> Self {
        U32Field::from_usize(n)
    }
}

trait FromUsize {
    fn from_usize(value: usize) -> Self;
}

impl FromUsize for U32Field {
    fn from_usize(value: usize) -> Self {
        U32Field::from_usize(value)
    }
}