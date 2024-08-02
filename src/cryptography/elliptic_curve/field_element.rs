use std::ops::{Add, Sub, Mul, Div, Rem};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldElement<T>
where
    T: PartialOrd + Add<Output = T> + Copy,
{
    pub num: T,
    pub prime: T,
}

impl<T> FieldElement<T>
where
    T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy,
{
    pub fn new(num: T, prime: T) -> Result<Self, String> {
        if num >= prime {
            return Err(format!("Num {:?} not in field range 0 to {:?}", num, prime));
        }
        Ok(FieldElement { num, prime })
    }

    pub fn pow(self, exponent: T) -> Self {
        let zero = self.prime - self.prime;
        let one = self.prime / self.prime;

        let mut result = Self::new(one, self.prime).unwrap();
        let mut counter = exponent % (self.prime - one);

        while counter > zero {
            result = result * self;
            counter = counter - one;
        }

        result
    }
}

impl<T> Add for FieldElement<T>
where
T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy,
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        if self.prime != other.prime {
            panic!("Primes must be the same");
        }
        if self.num + other.num >= self.prime {
            Self::new(self.num + other.num - self.prime, self.prime).unwrap()
        } else {
            Self::new(self.num + other.num, self.prime).unwrap()
        }
    }
}

impl<T> Sub for FieldElement<T>
where
T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if self.prime != other.prime {
            panic!("Cannot subtract two numbers in different Fields");
        }
        if self.num < other.num {
            Self::new(self.prime + self.num - other.num, self.prime).unwrap()
        } else {
            Self::new(self.num - other.num, self.prime).unwrap()
        }
    }
}

impl<T> Mul for FieldElement<T>
where
T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        if self.prime != other.prime {
            panic!("Cannot multiply two numbers in different Fields");
        }
        let zero = self.prime - self.prime;
        let one = self.prime / self.prime;

        let mut counter = other.num;
        let mut result = Self::new(zero, self.prime).unwrap();

        while counter > zero {
            result = result + self;
            counter = counter - one;
        }

        result
    }
}

impl<T> Div for FieldElement<T>
where
T: PartialOrd + Debug + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        let p = self.prime;
        let one = self.prime / self.prime;
        self * FieldElement::new(other.num, p).unwrap().pow(p - one - one)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use k256::elliptic_curve::rand_core::le;
    use primitive_types::U256;

    #[test]
    fn field_element_new() {
        let field_element = FieldElement::new(1, 5).unwrap();

        assert_eq!(field_element.num, 1);
        assert_eq!(field_element.prime, 5);
    }

    #[test]
    fn field_element_new_invalid() {
        let field_element = FieldElement::new(5, 5);

        assert_eq!(field_element, Err("Num 5 not in field range 0 to 5".to_string()));
    }

    #[test]
    fn eq() {
        let a = FieldElement::new(U256::from(2), U256::from(3));
        let b = FieldElement::new(U256::from(2), U256::from(3));
        let c = FieldElement::new(U256::from(1), U256::from(3));

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn add() {
        let a = FieldElement::new(U256::from(2), U256::from(7)).unwrap();
        let b = FieldElement::new(U256::from(1), U256::from(7)).unwrap();
        let c = FieldElement::new(U256::from(3), U256::from(7)).unwrap();

        assert_eq!(a + b, c);
    }

    #[test]
    fn sub() {
        let a = FieldElement::new(U256::from(6), U256::from(7)).unwrap();
        let b = FieldElement::new(U256::from(4), U256::from(7)).unwrap();
        let c = FieldElement::new(U256::from(2), U256::from(7)).unwrap();

        assert_eq!(a - b, c);
    }

    #[test]
    fn mul() {
        let a = FieldElement::new(U256::from(3), U256::from(13)).unwrap();
        let b = FieldElement::new(U256::from(12), U256::from(13)).unwrap();
        let c = FieldElement::new(U256::from(10), U256::from(13)).unwrap();

        assert_eq!(a * b, c);
    }

    #[test]
    fn div() {
        let a = FieldElement::new(U256::from(7), U256::from(19)).unwrap();
        let b = FieldElement::new(U256::from(5), U256::from(19)).unwrap();
        let c = FieldElement::new(U256::from(9), U256::from(19)).unwrap();

        assert_eq!(a / b, c);
    }

    #[test]
    fn pow() {
        let a = FieldElement::new(U256::from(3), U256::from(13)).unwrap();
        let b = FieldElement::new(U256::from(1), U256::from(13)).unwrap();

        assert_eq!(a.pow(U256::from(3)), b);
    }
}