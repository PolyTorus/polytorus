use std::ops::Add;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldElement<T>
where
    T: Add<Output = T>,
{
    pub num: T,
    pub prime: T,
}

impl<T> FieldElement<T>
where
    T: PartialOrd + Debug + Add<Output = T>,
{
    pub fn new(num: T, prime: T) -> Result<Self, String> {
        if num >= prime {
            return Err(format!("Num {:?} not in field range 0 to {:?}", num, prime));
        }
        Ok(FieldElement { num, prime })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}