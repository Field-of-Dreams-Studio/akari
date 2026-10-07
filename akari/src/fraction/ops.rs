use crate::fraction::definition::Fraction;
use core::fmt;
use core::num::NonZeroU64;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

impl Default for Fraction {
    fn default() -> Self {
        Self {
            numer: 0,
            denom: NonZeroU64::new(1).expect("one is non-zero"),
        }
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let lhs = self.numer as i128 * other.denom.get() as i128;
        let rhs = other.numer as i128 * self.denom.get() as i128;
        lhs.cmp(&rhs)
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denom.get() == 1 {
            write!(f, "{}", self.numer)
        } else {
            write!(f, "{}/{}", self.numer, self.denom.get())
        }
    }
}

impl Add for Fraction {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
            .expect("fraction addition overflowed its representation")
    }
}

impl Sub for Fraction {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("fraction subtraction overflowed its representation")
    }
}

impl Mul for Fraction {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs)
            .expect("fraction multiplication overflowed its representation")
    }
}

impl Div for Fraction {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs)
            .expect("fraction division by zero or overflow")
    }
}

impl Rem for Fraction {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        let div = self / rhs;
        let int_part = (div.numer as i128 / div.denom.get() as i128) as i64;
        self - (rhs * Fraction::from_integer(int_part))
    }
}

impl Neg for Fraction {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            numer: self
                .numer
                .checked_neg()
                .expect("fraction negation overflowed its numerator"),
            denom: self.denom,
        }
    }
}

impl AddAssign for Fraction {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Fraction {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for Fraction {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for Fraction {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl RemAssign for Fraction {
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl From<i64> for Fraction {
    fn from(n: i64) -> Self {
        Self::from_integer(n)
    }
}

impl From<i32> for Fraction {
    fn from(n: i32) -> Self {
        Self::from_integer(n as i64)
    }
}

impl From<(i64, i64)> for Fraction {
    fn from((numer, denom): (i64, i64)) -> Self {
        Self::new(numer, denom)
    }
}

impl From<(i64, NonZeroU64)> for Fraction {
    fn from((numer, denom): (i64, NonZeroU64)) -> Self {
        Self::new_nonzero(numer, denom)
    }
}
