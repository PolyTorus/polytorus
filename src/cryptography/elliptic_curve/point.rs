#[derive(Clone, PartialEq, Eq)]
pub enum Point<T> where T: std::fmt::Debug {
    Coordinate {
        x: T,
        y: T,
        a: T,
        b: T,
    },
    Infinity,
}

impl<T> std::fmt::Debug for Point<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Point::Coordinate { x, y, a, b } => write!(f, "Point({:?}, {:?})_({:?}, {:?})", x, y, a, b),
            Point::Infinity => write!(f, "Point(Infinity)"),
        }
    }
}

impl<T> Point<T>
where
    T: std::ops::Add<Output = T> + std::ops::Mul<Output = T> + std::fmt::Debug + PartialEq + Clone,
{
    pub fn new(x: T, y: T, a: T, b: T) -> Result<Self, String> {
        if y.clone() != x.clone() * x.clone() * x.clone() + a.clone() * x.clone() + b.clone() {
            return Err(format!("Point({:?}, {:?}) is not on the curve y^2 = x^3 + ax + b", x, y));
        }
        Ok(Point::Coordinate { x, y, a, b })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;

    #[test]
    fn point_new() {
        let x = U256::from(192);
        let y = U256::from(105);
        let a = U256::from(0);
        let b = U256::from(7);

        let point = Point::new(x, y, a, b).unwrap();

        assert_eq!(point, Point::Coordinate { x, y, a, b });
    }

    #[test]
    fn point_eq() {
        let a = Point::new(U256::from(18), U256::from(77), U256::from(5), U256::from(7));
        let b = Point::new(U256::from(18), U256::from(77), U256::from(5), U256::from(7));

        assert!(a == b);
    }
}
