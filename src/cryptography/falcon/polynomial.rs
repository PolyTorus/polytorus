use num::{One, Zero};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;
use std::default::Default;
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};
use itertools::Itertools;
use super::inverse::Inverse;