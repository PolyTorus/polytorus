use super::point::Point;
use std::ops::{Add, Sub, Mul, Div};

impl<T> Add for Point<T>
where
    T: PartialEq + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + Copy + std::fmt::Debug,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (
                Point::Coordinate { x: x0, y: y0, a: a0, b: b0 },
                Point::Coordinate { x: x1, y: y1, a: a1, b: b1 },
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