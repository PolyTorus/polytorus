#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Point<T>
where
    T: std::fmt::Debug,
{
    Coordinate { x: T, y: T, a: T, b: T },
    Infinity,
}

impl<T> std::fmt::Debug for Point<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Point::Coordinate { x, y, a, b } => {
                write!(f, "Point({:?}, {:?})_({:?}, {:?})", x, y, a, b)
            }
            Point::Infinity => write!(f, "Point(Infinity)"),
        }
    }
}

impl<T> Point<T>
where
    T: std::ops::Add<Output = T> + std::ops::Mul<Output = T> + std::fmt::Debug + PartialEq + Clone,
{
    pub fn new(x: T, y: T, a: T, b: T) -> Self {
        if y.clone() * y.clone()
            != x.clone() * x.clone() * x.clone() + a.clone() * x.clone() + b.clone()
        {
            panic!(
                "({:?}, {:?}) is not on the curve y^2 = x^3 + {:?}x + {:?}",
                x, y, a, b
            );
        }
        Point::Coordinate { x, y, a, b }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;

    #[test]
    fn point_new() {
        let _ = Point::new(U256::from(18), U256::from(77), U256::from(5), U256::from(7));
    }

    #[test]
    fn point_eq() {
        let a = Point::new(U256::from(18), U256::from(77), U256::from(5), U256::from(7));
        let b = Point::new(U256::from(18), U256::from(77), U256::from(5), U256::from(7));

        assert!(a == b);
    }
}
