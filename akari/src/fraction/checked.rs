use crate::fraction::definition::Fraction;

impl Fraction {
    /// Checked addition. Returns `None` on overflow or divide-by-zero.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let n = self
            .numer
            .checked_mul(rhs.denom)?
            .checked_add(rhs.numer.checked_mul(self.denom)?)?;
        let d = self.denom.checked_mul(rhs.denom)?;
        Some(Self::new(n, d))
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let n = self
            .numer
            .checked_mul(rhs.denom)?
            .checked_sub(rhs.numer.checked_mul(self.denom)?)?;
        let d = self.denom.checked_mul(rhs.denom)?;
        Some(Self::new(n, d))
    }

    /// Checked multiplication.
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let n = self.numer.checked_mul(rhs.numer)?;
        let d = self.denom.checked_mul(rhs.denom)?;
        Some(Self::new(n, d))
    }

    /// Checked division. Returns `None` if rhs numerator is 0.
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.numer == 0 {
            return None;
        }
        let n = self.numer.checked_mul(rhs.denom)?;
        let d = self.denom.checked_mul(rhs.numer)?;
        Some(Self::new(n, d))
    }
}
