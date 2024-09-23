use super::point::Point;
use std::ops::{Add, Div, Mul, Sub};

impl<T> Add for Point<T>
where
    T: PartialEq
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Copy
        + std::fmt::Debug,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (
                Point::Coordinate {
                    x: x0,
                    y: y0,
                    a: a0,
                    b: b0,
                },
                Point::Coordinate {
                    x: x1,
                    y: y1,
                    a: a1,
                    b: b1,
                },
            ) => {
                if a0 != a1 || b0 != b1 {
                    panic!("Points are not on the same curve")
                }
                if x0 == x1 {
                    if y0 == y1 - y1 {
                        return Point::Infinity;
                    }
                    let one = a0 / a0;
                    let two = one + one;
                    let three = one + one + one;
                    let s = (three + x0 * x0 + a0) / (two * y0);
                    let x2 = s * s - two * x0;

                    return Point::Coordinate {
                        x: x2,
                        y: s * (x0 - x2) - y0,
                        a: a0,
                        b: b0,
                    };
                }
                let s = (y1 - y0) / (x1 - x0);

                let x2 = s * s - x1 - x0;
                let y2 = s * (x0 - x2) - y0;

                Point::Coordinate {
                    x: x2,
                    y: y2,
                    a: a0,
                    b: b0,
                }
            }
            (Point::Coordinate { x, y, a, b }, Point::Infinity) => Point::Coordinate { x, y, a, b },
            (Point::Infinity, Point::Coordinate { x, y, a, b }) => Point::Coordinate { x, y, a, b },
            (Point::Infinity, Point::Infinity) => Point::Infinity,
        }
    }
}

impl<T, U> Mul<U> for Point<T>
where
    T: Add<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + Mul<Output = T>
        + PartialOrd
        + Copy
        + std::fmt::Debug,
    U: Sub<Output = U> + Div<Output = U> + Mul<Output = U> + PartialOrd + Copy + std::fmt::Debug,
{
    type Output = Point<T>;

    fn mul(self, other: U) -> Self::Output {
        let zero = other - other;
        let one = other / other;

        let mut result = Point::Infinity;
        let mut counter = other;

        while counter > zero {
            result = result + self;
            counter = counter - one;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::field_element::FieldElement;
    use super::super::point::Point;
    use primitive_types::U256;

    #[test]
    fn point_on_elliptic_curve() {
        let a = FieldElement::new(U256::from(0), U256::from(223)).unwrap();
        let b = FieldElement::new(U256::from(7), U256::from(223)).unwrap();
        let x = FieldElement::new(U256::from(192), U256::from(223)).unwrap();
        let y = FieldElement::new(U256::from(105), U256::from(223)).unwrap();

        assert_eq!(y * y, x * x * x + a * x + b);
    }

    #[test]
    fn add_points() {
        let a = FieldElement::new(U256::from(0), U256::from(223)).unwrap();
        let b = FieldElement::new(U256::from(7), U256::from(223)).unwrap();
        let x1 = FieldElement::new(U256::from(192), U256::from(223)).unwrap();
        let y1 = FieldElement::new(U256::from(105), U256::from(223)).unwrap();
        let x2 = FieldElement::new(U256::from(17), U256::from(223)).unwrap();
        let y2 = FieldElement::new(U256::from(56), U256::from(223)).unwrap();

        let p1 = Point::Coordinate { x: x1, y: y1, a, b };
        let p2 = Point::Coordinate { x: x2, y: y2, a, b };

        let p3 = p1 + p2;

        let x3 = FieldElement::new(U256::from(170), U256::from(223)).unwrap();
        let y3 = FieldElement::new(U256::from(142), U256::from(223)).unwrap();

        assert_eq!(p3, Point::Coordinate { x: x3, y: y3, a, b });
    }

    #[test]
    fn mul_point() {
        let p0 = Point::new(2, 5, 5, 7);
        let p1 = Point::new(2, -5, 5, 7);

        assert_ne!(p0, p1);
        assert_eq!(p0 * 3, p1);
        assert_eq!(p0 * U256::from(3), p1);
    }
}
