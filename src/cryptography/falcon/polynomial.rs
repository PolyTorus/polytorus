use num::{One, Zero};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;
use std::default::Default;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};
use itertools::Itertools;
use super::inverse::Inverse;
use super::field::{Felt, Q};

#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct Polynomial<F> {
    pub coefficients: Vec<F>,
}

impl<F> Polynomial<F>
where
    F: Clone,
{
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }
}

impl<F> Polynomial<F>
where
    F: Clone + Neg<Output = F>,
{
    #[allow(dead_code)]
    pub fn hermitian_adjoint(&self) -> Polynomial<F> {
        let coefficients = [
            vec![self.coefficients[0].clone()],
            self.coefficients
                .iter()
                .skip(1)
                .cloned()
                .map(|c| -c)
                .rev()
                .collect_vec(),
        ]
        .concat();
        Polynomial { coefficients }
    }
}

fn vector_karatsuba<
    F: Zero + AddAssign + Mul<Output = F> + Sub<Output = F> + Div<Output = F> + Clone,
>(
    left: &[F],
    right: &[F],
) -> Vec<F> {
    let n = left.len();
    if n <= 8 {
        let mut product = vec![F::zero(); left.len() + right.len() - 1];
        for (i, l) in left.iter().enumerate() {
            for (j, r) in right.iter().enumerate() {
                product[i + j] += l.clone() * r.clone();
            }
        }
        return product;
    }
    let n_over_2 = n / 2;
    let mut product = vec![F::zero(); 2 * n - 1];
    let left_lo = &left[0..n_over_2];
    let right_lo = &right[0..n_over_2];
    let left_hi = &left[n_over_2..];
    let right_hi = &right[n_over_2..];
    let left_sum = left_lo
        .iter()
        .zip(left_hi)
        .map(|(a, b)| a.clone() + b.clone())
        .collect_vec();
    let right_sum = right_lo
        .iter()
        .zip(right_hi)
        .map(|(a, b)| a.clone() + b.clone())
        .collect_vec();

    let prod_lo = vector_karatsuba(left_lo, right_lo);
    let prod_hi = vector_karatsuba(left_hi, right_hi);
    let prod_mid = vector_karatsuba(&left_sum, &right_sum)
        .iter()
        .zip(prod_lo.iter().zip(prod_hi.iter()))
        .map(|(s, (l, h))| s.clone() - (l.clone() + h.clone()))
        .collect_vec();

    for (i, l) in prod_lo.into_iter().enumerate() {
        product[i] = l;
    }
    for (i, m) in prod_mid.into_iter().enumerate() {
        product[i + n_over_2] += m;
    }
    for (i, h) in prod_hi.into_iter().enumerate() {
        product[i + n] += h
    }
    product
}

#[allow(private_bounds)] 
impl<F: Mul<Output = F> + Sub<Output = F> + AddAssign + Zero + Div<Output = F> + Inverse + Clone,> Polynomial<F>
{
    pub fn hadamard_mul(&self, other: &Self) -> Self {
        Polynomial::new(
            self.coefficients
                .iter()
                .zip(other.coefficients.iter())
                .map(|(a, b)| *a * *b)
                .collect_vec(),
        )
    }
    pub fn hadamard_div(&self, other: &Self) -> Self {
        let other_coefficients_inverse = F::batch_inverse_or_zero(&other.coefficients);
        Polynomial::new(
            self.coefficients
                .iter()
                .zip(other_coefficients_inverse.iter())
                .map(|(a, b)| *a * *b)
                .collect_vec(),
        )
    }

    pub fn hadamard_inv(&self) -> Self {
        let coefficients_inverse = F::batch_inverse_or_zero(&self.coefficients);
        Polynomial::new(coefficients_inverse)
    }
}

impl<F: Mul<Output = F> + Sub<Output = F> + AddAssign + Zero + Div<Output = F> + Clone>
    Polynomial<F>
{
    pub fn karatsuba(&self, other: &Self) -> Self {
        Polynomial::new(vector_karatsuba(&self.coefficients, &other.coefficients))
    }
}

impl<F: Zero + PartialEq + Clone> Polynomial<F> {
    pub fn degree(&self) -> Option<usize> {
        if self.coefficients.is_empty() {
            return None;
        }
        let mut max_index = self.coefficients.len() - 1;
        while self.coefficients[max_index] == F::zero() {
            if let Some(new_index) = max_index.checked_sub(1) {
                max_index = new_index;
            } else {
                return None;
            }
        }
        Some(max_index)
    }
    pub fn lc(&self) -> F {
        match self.degree() {
            Some(non_negative_degree) => self.coefficients[non_negative_degree].clone(),
            None => F::zero(),
        }
    }
}

impl<F: Zero + Clone> Polynomial<F> {
    pub fn shift(&self, shamt: usize) -> Self {
        Self {
            coefficients: [vec![F::zero(); shamt], self.coefficients.clone()].concat(),
        }
    }

    pub fn constant(f: F) -> Self {
        Self {
            coefficients: vec![f],
        }
    }

    pub fn map<G: Clone, C: FnMut(&F) -> G>(&self, closure: C) -> Polynomial<G> {
        Polynomial::<G>::new(self.coefficients.iter().map(closure).collect_vec())
    }

    pub fn fold<G, C: FnMut(G, &F) -> G + Clone>(&self, mut initial_value: G, closure: C) -> G {
        for c in self.coefficients.iter() {
            initial_value = (closure.clone())(initial_value, c);
        }
        initial_value
    }
}

impl<
        F: One + Zero + Clone + Neg<Output = F> + MulAssign + AddAssign + Sub<Output = F> + PartialEq,
    > Polynomial<F>
{
    pub fn reduce_by_cyclotomic(&self, n: usize) -> Self {
        let mut coefficients = vec![F::zero(); n];
        let mut sign = -F::one();
        for (i, c) in self.coefficients.iter().cloned().enumerate() {
            if i % n == 0 {
                sign *= -F::one();
            }
            coefficients[i % n] += sign.clone() * c;
        }
        Polynomial::new(coefficients)
    }
}

impl<
        F: One
            + Zero
            + Clone
            + Neg<Output = F>
            + MulAssign
            + AddAssign
            + Div<Output = F>
            + Sub<Output = F>
            + PartialEq,
    > Polynomial<F>
{
    pub fn cyclotomic_ring_inverse(&self, n: usize) -> Self {
        let mut cyclotomic_coefficients = vec![F::zero(); n + 1];
        cyclotomic_coefficients[0] = F::one();
        cyclotomic_coefficients[n] = F::one();
        let (_, a, _) = Polynomial::xgcd(self, &Polynomial::new(cyclotomic_coefficients));
        a
    }

    pub fn field_norm(&self) -> Self {
        let n = self.coefficients.len();
        let mut f0_coefficients = vec![F::zero(); n / 2];
        let mut f1_coefficients = vec![F::zero(); n / 2];
        for i in 0..n / 2 {
            f0_coefficients[i] = self.coefficients[2 * i].clone();
            f1_coefficients[i] = self.coefficients[2 * i + 1].clone();
        }
        let f0 = Polynomial::new(f0_coefficients);
        let f1 = Polynomial::new(f1_coefficients);
        let f0_squared = (f0.clone() * f0).reduce_by_cyclotomic(n / 2);
        let f1_squared = (f1.clone() * f1).reduce_by_cyclotomic(n / 2);
        let x = Polynomial::new(vec![F::zero(), F::one()]);
        f0_squared - (x * f1_squared).reduce_by_cyclotomic(n / 2)
    }

    pub fn lift_next_cyclotomic(&self) -> Self {
        let n = self.coefficients.len();
        let mut coefficients = vec![F::zero(); n * 2];
        for i in 0..n {
            coefficients[2 * i] = self.coefficients[i].clone();
        }
        Self::new(coefficients)
    }

    pub fn galois_adjoint(&self) -> Self {
        Self::new(
            self.coefficients
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if i % 2 == 0 {
                        c.clone()
                    } else {
                        c.clone().neg()
                    }
                })
                .collect_vec(),
        )
    }
}

impl<
        F: One
            + Zero
            + Clone
            + Neg<Output = F>
            + MulAssign
            + AddAssign
            + Div<Output = F>
            + Sub<Output = F>
            + PartialEq,
    > Polynomial<F>
{
    pub fn xgcd(a: &Self, b: &Self) -> (Self, Self, Self) {
        if a.is_zero() || b.is_zero() {
            return (Self::zero(), Self::zero(), Self::zero());
        }
        let (mut old_r, mut r) = (a.clone(), b.clone());
        let (mut old_s, mut s) = (Self::one(), Self::zero());
        let (mut old_t, mut t) = (Self::zero(), Self::one());

        while !r.is_zero() {
            let quotient = old_r.clone() / r.clone();
            (old_r, r) = (r.clone(), old_r.clone() - quotient.clone() * r.clone());
            (old_s, s) = (s.clone(), old_s.clone() - quotient.clone() * s.clone());
            (old_t, t) = (t.clone(), old_t.clone() - quotient.clone() * t.clone());
        }

        (old_r, old_s, old_t)
    }
}

impl<F: Clone + Into<f64>> Polynomial<F> {
    #[allow(dead_code)]
    pub(crate) fn l2_norm(&self) -> f64 {
        self.coefficients
            .iter()
            .map(|i| Into::<f64>::into(i.clone()))
            .map(|i| i * i)
            .sum::<f64>()
            .sqrt()
    }
    pub fn l2_norm_squared(&self) -> f64 {
        self.coefficients
            .iter()
            .map(|i| Into::<f64>::into(i.clone()))
            .map(|i| i * i)
            .sum::<f64>()
    }
}

impl<F> PartialEq for Polynomial<F>
where
    F: Zero + PartialEq + Clone + AddAssign,
{
    fn eq(&self, other: &Self) -> bool {
        if self.is_zero() && other.is_zero() {
            true
        } else if self.is_zero() || other.is_zero() {
            false
        } else {
            let self_degree = self.degree().unwrap();
            let other_degree = other.degree().unwrap();
            self.coefficients[0..=self_degree] == other.coefficients[0..=other_degree]
        }
    }
}

impl<F> Eq for Polynomial<F> where F: Zero + PartialEq + Clone + AddAssign {}

impl<F> Add for &Polynomial<F>
where
    F: Add<Output = F> + AddAssign + Clone,
{
    type Output = Polynomial<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let coefficients = if self.coefficients.len() >= rhs.coefficients.len() {
            let mut coefficients = self.coefficients.clone();
            for (i, c) in rhs.coefficients.iter().enumerate() {
                coefficients[i] += c.clone();
            }
            coefficients
        } else {
            let mut coefficients = rhs.coefficients.clone();
            for (i, c) in self.coefficients.iter().enumerate() {
                coefficients[i] += c.clone();
            }
            coefficients
        };
        Self::Output { coefficients }
    }
}

impl<F> Add for Polynomial<F>
where
    F: Add<Output = F> + AddAssign + Clone,
{
    type Output = Polynomial<F>;
    fn add(self, rhs: Self) -> Self::Output {
        let coefficients = if self.coefficients.len() >= rhs.coefficients.len() {
            let mut coefficients = self.coefficients.clone();
            for (i, c) in rhs.coefficients.into_iter().enumerate() {
                coefficients[i] += c;
            }
            coefficients
        } else {
            let mut coefficients = rhs.coefficients.clone();
            for (i, c) in self.coefficients.into_iter().enumerate() {
                coefficients[i] += c;
            }
            coefficients
        };
        Self::Output { coefficients }
    }
}

impl<F> AddAssign for Polynomial<F>
where
    F: Add<Output = F> + AddAssign + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        if self.coefficients.len() >= rhs.coefficients.len() {
            for (i, c) in rhs.coefficients.into_iter().enumerate() {
                self.coefficients[i] += c;
            }
        } else {
            let mut coefficients = rhs.coefficients.clone();
            for (i, c) in self.coefficients.iter().enumerate() {
                coefficients[i] += c.clone();
            }
            self.coefficients = coefficients;
        }
    }
}

impl<F> Sub for &Polynomial<F>
where
    F: Sub<Output = F> + Clone + Neg<Output = F> + Add<Output = F> + AddAssign,
{
    type Output = Polynomial<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        self + &(-rhs)
    }
}
impl<F> Sub for Polynomial<F>
where
    F: Sub<Output = F> + Clone + Neg<Output = F> + Add<Output = F> + AddAssign,
{
    type Output = Polynomial<F>;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl<F> SubAssign for Polynomial<F>
where
    F: Add<Output = F> + Neg<Output = F> + AddAssign + Clone + Sub<Output = F>,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.coefficients = self.clone().sub(rhs).coefficients;
    }
}

impl<F: Neg<Output = F> + Clone> Neg for &Polynomial<F> {
    type Output = Polynomial<F>;

    fn neg(self) -> Self::Output {
        Self::Output {
            coefficients: self.coefficients.iter().cloned().map(|a| -a).collect(),
        }
    }
}
impl<F: Neg<Output = F> + Clone> Neg for Polynomial<F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            coefficients: self.coefficients.iter().cloned().map(|a| -a).collect(),
        }
    }
}

impl<F> Mul for &Polynomial<F>
where
    F: Add + AddAssign + Mul<Output = F> + Sub<Output = F> + Zero + PartialEq + Clone,
{
    type Output = Polynomial<F>;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return Polynomial::<F>::zero();
        }
        let mut coefficients =
            vec![F::zero(); self.coefficients.len() + other.coefficients.len() - 1];
        for i in 0..self.coefficients.len() {
            for j in 0..other.coefficients.len() {
                coefficients[i + j] += self.coefficients[i].clone() * other.coefficients[j].clone();
            }
        }
        Polynomial { coefficients }
    }
}

impl<F> Mul for Polynomial<F>
where
    F: Add + AddAssign + Mul<Output = F> + Zero + PartialEq + Clone,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let mut coefficients =
            vec![F::zero(); self.coefficients.len() + other.coefficients.len() - 1];
        for i in 0..self.coefficients.len() {
            for j in 0..other.coefficients.len() {
                coefficients[i + j] += self.coefficients[i].clone() * other.coefficients[j].clone();
            }
        }
        Self { coefficients }
    }
}

impl<F: Add + Mul<Output = F> + Zero + Clone> Mul<F> for &Polynomial<F> {
    type Output = Polynomial<F>;

    fn mul(self, other: F) -> Self::Output {
        Polynomial {
            coefficients: self
                .coefficients
                .iter()
                .cloned()
                .map(|i| i * other.clone())
                .collect_vec(),
        }
    }
}
impl<F: Add + Mul<Output = F> + Zero + Clone> Mul<F> for Polynomial<F> {
    type Output = Polynomial<F>;

    fn mul(self, other: F) -> Self::Output {
        Polynomial {
            coefficients: self
                .coefficients
                .iter()
                .cloned()
                .map(|i| i * other.clone())
                .collect_vec(),
        }
    }
}
impl<F> One for Polynomial<F>
where
    F: Clone + One + PartialEq + Zero + AddAssign,
{
    fn one() -> Self {
        Self {
            coefficients: vec![F::one()],
        }
    }
}

impl<F> Zero for Polynomial<F>
where
    F: Zero + PartialEq + Clone + AddAssign,
{
    fn zero() -> Self {
        Self {
            coefficients: vec![],
        }
    }

    fn is_zero(&self) -> bool {
        self.degree().is_none()
    }
}

impl<F> Div<Polynomial<F>> for Polynomial<F>
where
    F: Zero
        + One
        + PartialEq
        + AddAssign
        + Clone
        + Mul<Output = F>
        + MulAssign
        + Div<Output = F>
        + Neg<Output = F>
        + Sub<Output = F>,
{
    type Output = Polynomial<F>;

    fn div(self, denominator: Self) -> Self::Output {
        if denominator.is_zero() {
            panic!();
        }
        if self.is_zero() {
            Self::zero();
        }
        let mut remainder = self.clone();
        let mut quotient = Polynomial::<F>::zero();
        while remainder.degree().unwrap() >= denominator.degree().unwrap() {
            let shift = remainder.degree().unwrap() - denominator.degree().unwrap();
            let quotient_coefficient = remainder.lc() / denominator.lc();
            let monomial = Self::constant(quotient_coefficient).shift(shift);
            quotient += monomial.clone();
            remainder -= monomial * denominator.clone();
            if remainder.is_zero() {
                break;
            }
        }
        quotient
    }
}


pub fn hash_to_point(string: &[u8], n: usize) -> Polynomial<Felt> {
    const K: u32 = (1u32 << 16) / Q;

    let mut hasher = Shake256::default();
    hasher.update(string);
    let mut reader = hasher.finalize_xof();

    let mut coefficients: Vec<Felt> = vec![];
    while coefficients.len() != n {
        let mut randomness = [0u8; 2];
        reader.read(&mut randomness);
        // Arabic endianness but so be it
        let t = ((randomness[0] as u32) << 8) | (randomness[1] as u32);
        if t < K * Q {
            coefficients.push(Felt::new((t % Q) as i16));
        }
    }

    Polynomial { coefficients }
}

impl<T: Display> Display for Polynomial<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.coefficients.iter().join(", "))
    }
}
