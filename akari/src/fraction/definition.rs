/// Represents an exact rational fraction `numerator / denominator`.
///
/// Under the hood, fractions are kept reduced to lowest terms, with the denominator
/// guaranteed to be strictly positive (`> 0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fraction {
    pub(crate) numer: i64,
    pub(crate) denom: i64,
}
