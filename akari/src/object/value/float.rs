//! Small `f64` helpers exposed as a trait, so call sites stay uniform across
//! std and no_std builds.
//!
//! `core` doesn't expose `f64::ceil`, `f64::powi`, etc. — those live in
//! `std::f64`. This module provides the [`FloatExt`] trait with `2`-suffixed
//! method names (`ceil2`, `powi2`, …) that delegate to the inherent methods
//! in std and to manual implementations in no_std.

/// Extension trait providing `f64` math methods that don't live in `core`.
///
/// Each method is `2`-suffixed to avoid shadowing the inherent `f64` methods
/// in std builds.
pub trait FloatExt {
    /// Fractional part of `self`. Uses `f64::fract` in std; in no_std,
    /// computed as `self - (self as i64 as f64)`.
    fn fract2(&self) -> f64;

    /// Smallest integer ≥ `self`, for finite values within `i64` range.
    fn ceil2(&self) -> f64;

    /// Largest integer ≤ `self`, for finite values within `i64` range.
    fn floor2(&self) -> f64;

    /// `self^exp` via binary exponentiation.
    fn powi2(&self, exp: i32) -> f64;

    /// `self^exp` for general `f64` exponent.
    ///
    /// In std this delegates to [`f64::powf`].
    ///
    /// # TODO — `no_std` is a placeholder
    ///
    /// `core` has no transcendentals. Until akari grows a real `powf`
    /// (Taylor / Padé inline, or opt-in `libm`), this rounds `exp` to
    /// the nearest integer and routes through [`FloatExt::powi2`]:
    ///
    /// - Integer exponents (`2.0.powf2(3.0)`) — exact.
    /// - Nearly-integer exponents (`2.0.powf2(3.0001)`) — exact for the
    ///   intended power.
    /// - Genuinely fractional exponents — **silently lossy**:
    ///   `2.0.powf2(0.5)` returns `2.0`, not `√2`.
    ///
    /// Gate on `exp.fract2() == 0.0` upstream and surface an error if
    /// your `no_std` workload computes non-integer powers.
    fn powf2(&self, exp: f64) -> f64;
}

impl FloatExt for f64 {
    #[inline]
    fn fract2(&self) -> f64 {
        #[cfg(feature = "no_std")]
        {
            *self - (*self as i64 as f64)
        }
        #[cfg(not(feature = "no_std"))]
        {
            self.fract()
        }
    }

    #[inline]
    fn ceil2(&self) -> f64 {
        #[cfg(feature = "no_std")]
        {
            let t = *self as i64 as f64;
            if t < *self { t + 1.0 } else { t }
        }
        #[cfg(not(feature = "no_std"))]
        {
            self.ceil()
        }
    }

    #[inline]
    fn floor2(&self) -> f64 {
        #[cfg(feature = "no_std")]
        {
            let t = *self as i64 as f64;
            if t > *self { t - 1.0 } else { t }
        }
        #[cfg(not(feature = "no_std"))]
        {
            self.floor()
        }
    }

    #[inline]
    fn powi2(&self, exp: i32) -> f64 {
        #[cfg(feature = "no_std")]
        {
            let mut result = 1.0_f64;
            let mut b = *self;
            let mut n = exp.unsigned_abs();
            while n > 0 {
                if n & 1 == 1 {
                    result *= b;
                }
                b *= b;
                n >>= 1;
            }
            if exp < 0 { 1.0 / result } else { result }
        }
        #[cfg(not(feature = "no_std"))]
        {
            self.powi(exp)
        }
    }

    #[inline]
    fn powf2(&self, exp: f64) -> f64 {
        #[cfg(feature = "no_std")]
        {
            // TODO: real powf via `exp(exp * ln(self))` — roughly 50 lines
            // of Taylor/Padé approximation, or pull `libm` behind a feature
            // flag. Until then, round non-integer exponents to the nearest
            // integer and route through `powi2`. This is lossy for genuinely
            // fractional exponents (e.g. `2.0.powf2(0.5)` returns `2.0`, not
            // `√2`) but at least keeps the result finite.
            let rounded = if exp >= 0.0 { (exp + 0.5) as i32 } else { (exp - 0.5) as i32 };
            self.powi2(rounded)
        }
        #[cfg(not(feature = "no_std"))]
        {
            self.powf(exp)
        }
    }
}
