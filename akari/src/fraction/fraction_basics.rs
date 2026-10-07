/// # Fraction Type in Akari
///
/// This test script demonstrates and verifies how raw `Fraction` values work
/// in the Akari library. Fractions are exact rational numbers stored as
/// `numerator / denominator` without converting to floating-point.
///
/// Topics covered:
///  1. Construction and automatic reduction to lowest terms
///  2. Arithmetic (+, -, *, /, %, negation)
///  3. Ordering and comparison
///  4. Checked operations (overflow-safe)
///  5. Conversion to/from f64 and Display
///  6. Float-to-Fraction approximation (continued-fraction algorithm)
///
/// Run this with:
///   cargo test --test fraction_basics
use akari::Fraction;
use core::num::NonZeroU64;

// ─────────────────────────────────────────────────────────────────
// 1. CONSTRUCTION: How to create a Fraction
// ─────────────────────────────────────────────────────────────────

/// Fractions are automatically reduced to their simplest (lowest) form.
/// e.g. 4/8 becomes 1/2, 6/9 becomes 2/3, 3/1 stays 3.
#[test]
fn test_fraction_construction_and_reduction() {
    // Direct construction – always reduces to lowest terms
    let half = Fraction::new(1, 2);
    let four_eighths = Fraction::new(4, 8); // same as 1/2

    assert_eq!(half.numer(), 1);
    assert_eq!(half.denom(), 2);
    assert_eq!(half, four_eighths, "4/8 should reduce to 1/2");

    // 6/9  →  2/3
    let f = Fraction::new(6, 9);
    assert_eq!(f.numer(), 2);
    assert_eq!(f.denom(), 3);
    println!("6/9 reduced = {}", f); // prints "2/3"

    // A negative fraction keeps the sign on the numerator, denominator stays > 0
    let neg = Fraction::new(-3, 6);
    assert_eq!(neg.numer(), -1);
    assert_eq!(neg.denom(), 2);
    println!("-3/6 reduced = {}", neg); // prints "-1/2"

    // Constructing from an integer
    let three = Fraction::from_integer(3);
    assert!(three.is_integer()); // denominator is 1
    println!("integer 3 as fraction = {}", three); // prints "3"

    // Constructing from a tuple
    let from_tuple: Fraction = (7_i64, 4_i64).into();
    assert_eq!(from_tuple, Fraction::new(7, 4));
    println!("(7,4) as fraction = {}", from_tuple); // prints "7/4"
}

/// Division by zero is now a caller choice: use the fallible constructor
/// when you want to handle it explicitly.
#[test]
fn test_fraction_zero_denominator() {
    assert_eq!(Fraction::try_new(5, 0), None);
    assert_eq!(Fraction::try_new(5, 2), Some(Fraction::new(5, 2)));
}

/// The representation handles the complete signed-input range without calling
/// `i64::abs`, and exposes the positive denominator as an unsigned value.
#[test]
fn test_fraction_integer_boundaries_and_nonzero_denominator() {
    let min_numerator = Fraction::new(i64::MIN, 1);
    assert_eq!(min_numerator.numer(), i64::MIN);
    assert_eq!(min_numerator.denom(), 1);

    let min_denominator = Fraction::new(1, i64::MIN);
    assert_eq!(min_denominator.numer(), -1);
    assert_eq!(min_denominator.denom(), 1_u64 << 63);
    assert_eq!(
        min_denominator.denom_nonzero(),
        NonZeroU64::new(1_u64 << 63).unwrap()
    );

    // Normalizing i64::MIN / -1 would require the unavailable positive i64 value 2^63.
    assert_eq!(Fraction::try_new(i64::MIN, -1), None);

    let full_width_denom = Fraction::new_nonzero(1, NonZeroU64::new(u64::MAX).unwrap());
    assert_eq!(full_width_denom.numer(), 1);
    assert_eq!(full_width_denom.denom(), u64::MAX);

    let from_tuple: Fraction = (6, NonZeroU64::new(8).unwrap()).into();
    assert_eq!(from_tuple, Fraction::new(3, 4));
}

// ─────────────────────────────────────────────────────────────────
// 2. ARITHMETIC: Exact fraction math
// ─────────────────────────────────────────────────────────────────

/// Addition: a/b + c/d = (ad + bc) / bd, then reduced.
#[test]
fn test_fraction_addition() {
    let a = Fraction::new(1, 2); // 1/2
    let b = Fraction::new(1, 3); // 1/3
    let sum = a + b; // 1/2 + 1/3 = 5/6

    assert_eq!(sum, Fraction::new(5, 6));
    println!("{} + {} = {}", a, b, sum); // 1/2 + 1/3 = 5/6

    // Adding to an integer fraction
    let whole = Fraction::new(2, 1); // 2
    let result = whole + b; // 2 + 1/3 = 7/3
    assert_eq!(result, Fraction::new(7, 3));
    println!("{} + {} = {}", whole, b, result); // 2 + 1/3 = 7/3

    // Compound assignment +=
    let mut acc = Fraction::new(1, 4); // 1/4
    acc += Fraction::new(3, 4); // 1/4 + 3/4 = 1
    assert!(acc.is_integer());
    assert_eq!(acc.numer(), 1);
    println!("1/4 += 3/4 = {}", acc); // "1"
}

/// Subtraction: a/b - c/d = (ad - bc) / bd, then reduced.
#[test]
fn test_fraction_subtraction() {
    let a = Fraction::new(3, 4);
    let b = Fraction::new(1, 4);
    let diff = a - b;

    assert_eq!(diff, Fraction::new(1, 2));
    println!("{} - {} = {}", a, b, diff); // 3/4 - 1/4 = 1/2

    // Negative result
    let neg = Fraction::new(1, 6) - Fraction::new(1, 3);
    assert_eq!(neg, Fraction::new(-1, 6));
    println!("1/6 - 1/3 = {}", neg); // -1/6
}

/// Multiplication: (a/b) * (c/d) = ac / bd, then reduced.
#[test]
fn test_fraction_multiplication() {
    let a = Fraction::new(2, 3);
    let b = Fraction::new(3, 4);
    let product = a * b; // 2/3 * 3/4 = 6/12 = 1/2

    assert_eq!(product, Fraction::new(1, 2));
    println!("{} * {} = {}", a, b, product); // 2/3 * 3/4 = 1/2

    // By zero
    let zero = Fraction::new(0, 1);
    let zeroed = a * zero;
    assert_eq!(zeroed, Fraction::new(0, 1));
}

/// Division: (a/b) / (c/d) = (a/b) * (d/c) = ad / bc, then reduced.
#[test]
fn test_fraction_division() {
    let a = Fraction::new(1, 2);
    let b = Fraction::new(1, 4);
    let quot = a / b; // (1/2) / (1/4) = (1/2) * (4/1) = 4/2 = 2

    assert_eq!(quot, Fraction::new(2, 1));
    assert!(quot.is_integer());
    println!("{} / {} = {}", a, b, quot); // 1/2 / 1/4 = 2

    // Dividing by a fraction > 1
    let third = Fraction::new(1, 3);
    let two = Fraction::new(2, 1);
    println!("{} / {} = {}", third, two, third / two); // 1/3 / 2 = 1/6
    assert_eq!(third / two, Fraction::new(1, 6));
}

/// Remainder / modulo for fractions.
#[test]
fn test_fraction_remainder() {
    // 5/6 % 1/3  →  5/6 - floor(5/6 ÷ 1/3) * 1/3
    //             =  5/6 - floor(5/2) * 1/3
    //             =  5/6 - 2 * 1/3 = 5/6 - 2/3 = 1/6
    let a = Fraction::new(5, 6);
    let b = Fraction::new(1, 3);
    let r = a % b;
    assert_eq!(r, Fraction::new(1, 6));
    println!("{} % {} = {}", a, b, r); // 5/6 % 1/3 = 1/6
}

/// Negation.
#[test]
fn test_fraction_negation() {
    let a = Fraction::new(3, 5);
    let neg = -a;
    assert_eq!(neg, Fraction::new(-3, 5));
    assert_eq!(neg.numer(), -3);
    assert_eq!(neg.denom(), 5);
    println!("-({}) = {}", a, neg); // -(3/5) = -3/5
}

// ─────────────────────────────────────────────────────────────────
// 3. COMPARISON: Ordering without converting to f64
// ─────────────────────────────────────────────────────────────────

/// Fractions compare exactly using cross-multiplication so there is no
/// floating-point rounding error.
#[test]
fn test_fraction_ordering() {
    let a = Fraction::new(1, 3);
    let b = Fraction::new(1, 2);

    assert!(a < b, "1/3 < 1/2");
    assert!(b > a, "1/2 > 1/3");

    // Fractions equal after reduction
    let c = Fraction::new(2, 4); // 1/2
    assert_eq!(b, c);
    assert!(c >= a);
    assert!(a <= c);

    // Sorting a list of fractions
    let mut fracs = vec![
        Fraction::new(5, 6),
        Fraction::new(1, 4),
        Fraction::new(2, 3),
        Fraction::new(1, 2),
    ];
    fracs.sort();
    assert_eq!(fracs[0], Fraction::new(1, 4));
    assert_eq!(fracs[3], Fraction::new(5, 6));
    println!(
        "sorted: {:?}",
        fracs.iter().map(|f| f.to_string()).collect::<Vec<_>>()
    );
}

// ─────────────────────────────────────────────────────────────────
// 4. CHECKED OPERATIONS: Overflow-safe variants
// ─────────────────────────────────────────────────────────────────

/// `checked_*` methods return `Option<Fraction>` – `None` on overflow.
#[test]
fn test_checked_operations() {
    let a = Fraction::new(1, 2);
    let b = Fraction::new(1, 3);

    assert_eq!(a.checked_add(b), Some(Fraction::new(5, 6)));
    assert_eq!(a.checked_sub(b), Some(Fraction::new(1, 6)));
    assert_eq!(a.checked_mul(b), Some(Fraction::new(1, 6)));
    assert_eq!(a.checked_div(b), Some(Fraction::new(3, 2)));

    // Division by zero returns None
    let zero = Fraction::new(0, 1);
    assert_eq!(a.checked_div(zero), None);
    println!("checked_div by zero = {:?}", a.checked_div(zero)); // None

    // Intermediates use wider integers, so reducible results do not overflow early.
    assert_eq!(
        Fraction::new(i64::MAX, 2).checked_mul(Fraction::new(2, i64::MAX)),
        Some(Fraction::from_integer(1))
    );

    assert_eq!(
        Fraction::from_integer(i64::MAX).checked_add(Fraction::from_integer(1)),
        None
    );
}

// ─────────────────────────────────────────────────────────────────
// 5. CONVERSION: to_f64, Display, From<i32/i64>
// ─────────────────────────────────────────────────────────────────

#[test]
fn test_fraction_conversion() {
    let a = Fraction::new(1, 4);

    // To f64 – exact for power-of-two denominators
    assert!((a.to_f64() - 0.25).abs() < 1e-15);
    println!("1/4 as f64 = {}", a.to_f64()); // 0.25

    // Display: "n/d" when denom > 1, or just "n" when it's an integer
    assert_eq!(format!("{}", Fraction::new(3, 5)), "3/5");
    assert_eq!(format!("{}", Fraction::new(4, 2)), "2"); // reduces to 2/1
    assert_eq!(format!("{}", Fraction::new(-1, 3)), "-1/3");

    // From integer primitives
    let from_i32: Fraction = (2_i32).into();
    let from_i64: Fraction = (7_i64).into();
    assert_eq!(from_i32, Fraction::new(2, 1));
    assert_eq!(from_i64, Fraction::new(7, 1));
}

// ─────────────────────────────────────────────────────────────────
// 6. FLOAT-TO-FRACTION: Continued-fraction approximation
// ─────────────────────────────────────────────────────────────────
//
// The continued-fraction (CF) algorithm finds the simplest fraction
// p/q that exactly (or very closely) represents an f64. It works by
// expanding the float into a sequence of integer "quotients" and
// building convergents h/k. Each convergent is the best rational
// approximation for its denominator size.

/// `from_f64_bounded(f, max_denom)` — core CF algorithm with a custom denominator cap.
#[test]
fn test_from_f64_bounded() {
    // Exact half
    assert_eq!(
        Fraction::from_f64_bounded(1.5, 100),
        Some(Fraction::new(3, 2))
    );
    println!(
        "1.5  (max=100) → {:?}",
        Fraction::from_f64_bounded(1.5, 100)
    );

    // 0.1 representable as 1/10
    assert_eq!(
        Fraction::from_f64_bounded(0.1, 100),
        Some(Fraction::new(1, 10))
    );

    // Pi approximation: 22/7 is the best fit with denominator ≤ 10
    assert_eq!(
        Fraction::from_f64_bounded(3.14159, 10),
        Some(Fraction::new(22, 7))
    );
    println!(
        "3.14159 (max=10) → {:?}",
        Fraction::from_f64_bounded(3.14159, 10)
    );

    // Larger cap → closer approximation (355/113 is famous for pi)
    let pi_close = Fraction::from_f64_bounded(std::f64::consts::PI, 1000).unwrap();
    assert!((pi_close.to_f64() - std::f64::consts::PI).abs() < 1e-6);
    println!("pi (max=1000) → {}", pi_close); // 355/113

    // Whole number
    assert_eq!(
        Fraction::from_f64_bounded(5.0, 100),
        Some(Fraction::new(5, 1))
    );

    // NaN and infinity always return None
    assert_eq!(Fraction::from_f64_bounded(f64::NAN, 100), None);
    assert_eq!(Fraction::from_f64_bounded(f64::INFINITY, 100), None);
    assert_eq!(Fraction::from_f64_bounded(f64::NEG_INFINITY, 100), None);
    assert_eq!(Fraction::from_f64_bounded(0.5, 0), None);
    assert_eq!(Fraction::from_f64_bounded(0.5, -1), None);
}

/// `from_f64(f)` — same as `from_f64_bounded` with `max_denom = 1_000_000`.
/// Handles common decimal fractions exactly.
#[test]
fn test_from_f64() {
    // Integer-valued floats
    assert_eq!(Fraction::from_f64(0.0), Some(Fraction::new(0, 1)));
    assert_eq!(Fraction::from_f64(2.0), Some(Fraction::new(2, 1)));
    assert_eq!(Fraction::from_f64(-3.0), Some(Fraction::new(-3, 1)));

    // Common decimal fractions
    assert_eq!(Fraction::from_f64(0.25), Some(Fraction::new(1, 4)));
    assert_eq!(Fraction::from_f64(0.5), Some(Fraction::new(1, 2)));
    assert_eq!(Fraction::from_f64(0.75), Some(Fraction::new(3, 4)));
    assert_eq!(Fraction::from_f64(0.1), Some(Fraction::new(1, 10)));
    assert_eq!(Fraction::from_f64(1.5), Some(Fraction::new(3, 2)));
    assert_eq!(Fraction::from_f64(1.9), Some(Fraction::new(19, 10)));
    assert_eq!(Fraction::from_f64(0.125), Some(Fraction::new(1, 8)));

    // Repeating decimals: CF finds the clean fraction
    assert_eq!(Fraction::from_f64(1.0 / 3.0), Some(Fraction::new(1, 3)));
    assert_eq!(Fraction::from_f64(2.0 / 3.0), Some(Fraction::new(2, 3)));
    assert_eq!(Fraction::from_f64(1.0 / 7.0), Some(Fraction::new(1, 7)));
    println!("1/7 round-trip: {:?}", Fraction::from_f64(1.0 / 7.0));

    // Negative values
    assert_eq!(Fraction::from_f64(-0.5), Some(Fraction::new(-1, 2)));
    assert_eq!(Fraction::from_f64(-1.75), Some(Fraction::new(-7, 4)));

    // NaN → None
    assert_eq!(Fraction::from_f64(f64::NAN), None);

    // Round-trip property: from_f64(f).to_f64() ≈ f for common decimals
    let vals: &[f64] = &[0.1, 0.2, 0.3, 1.0 / 7.0, 22.0 / 7.0];
    for &v in vals {
        let frac = Fraction::from_f64(v).expect("should parse");
        let err = (frac.to_f64() - v).abs();
        assert!(
            err < 1e-9,
            "round-trip failed for {}: got {}, err={:.2e}",
            v,
            frac,
            err
        );
        println!("{} → {} (err = {:.2e})", v, frac, err);
    }
}

/// `from_f64_tol(f, tolerance)` — only returns `Some` when the best CF approximation
/// is within `tolerance` of the original float.
#[test]
fn test_from_f64_tol() {
    // 1/3 as float (0.3333333333333333) -> 1/3
    assert_eq!(
        Fraction::from_f64_tol(1.0 / 3.0, 0.01),
        Some(Fraction::new(1, 3))
    );
    println!(
        "1/3 (tol=0.01) → {:?}",
        Fraction::from_f64_tol(1.0 / 3.0, 0.01)
    );

    // 0.333 -> 333/1000 exactly
    assert_eq!(
        Fraction::from_f64_tol(0.333, 1e-4),
        Some(Fraction::new(333, 1000))
    );

    // Exact float: zero error – passes any tolerance
    assert_eq!(
        Fraction::from_f64_tol(0.5, 1e-15),
        Some(Fraction::new(1, 2))
    );

    // If tolerance is smaller than the approximation error for a hard float, returns None
    // (e.g. asking for pi/4 to be exact within 1e-15 with only denom≤1e6)
    // This is a property test — just verify it doesn't panic
    let _ = Fraction::from_f64_tol(std::f64::consts::PI, 1e-15);

    // NaN → None regardless of tolerance
    assert_eq!(Fraction::from_f64_tol(f64::NAN, 1.0), None);
}

/// `approx_f64(f)` — infallible: falls back to integer truncation for NaN/Inf.
#[test]
fn test_approx_f64() {
    // Normal values behave like from_f64
    assert_eq!(Fraction::approx_f64(1.5), Fraction::new(3, 2));
    assert_eq!(Fraction::approx_f64(0.1), Fraction::new(1, 10));
    assert_eq!(Fraction::approx_f64(0.25), Fraction::new(1, 4));
    assert_eq!(Fraction::approx_f64(3.0), Fraction::new(3, 1));

    // Negative
    assert_eq!(Fraction::approx_f64(-1.5), Fraction::new(-3, 2));
    assert_eq!(Fraction::approx_f64(-0.25), Fraction::new(-1, 4));

    // NaN fallback: `f64::NAN as i64` is 0 on most platforms → 0/1
    let nan_result = Fraction::approx_f64(f64::NAN);
    println!(
        "approx_f64(NaN) = {} (fallback to integer truncation)",
        nan_result
    );

    // Verify it never panics for any value, including edge cases
    let _ = Fraction::approx_f64(f64::MAX);
    let _ = Fraction::approx_f64(f64::MIN);
    let _ = Fraction::approx_f64(0.0);
    let _ = Fraction::approx_f64(-0.0);
}

// ─────────────────────────────────────────────────────────────────
// 7. 128-BIT INTERMEDIATE VALUES & OVERFLOW EXCEPTIONS
// ─────────────────────────────────────────────────────────────────

/// Intermediate arithmetic calculations can exceed 64-bit bounds (using 128-bit unsigned math),
/// but successfully reduce to valid 64-bit fractions via GCD reduction.
#[test]
fn test_intermediate_128bit_values_reduced() {
    // 1. Multiplication where intermediate numerator exceeds 64-bit i64::MAX (9_223_372_036_854_775_807),
    //    intermediate numer = i64::MAX * 2 = 18_446_744_073_709_551_614 (> 64-bit i64::MAX).
    //    Intermediate denom = 2 * i64::MAX = 18_446_744_073_709_551_614.
    //    GCD reduction reduces it to 1/1.
    let a = Fraction::new_nonzero(i64::MAX, NonZeroU64::new(2).unwrap());
    let b = Fraction::new(2, i64::MAX);
    assert_eq!(a * b, Fraction::new(1, 1));
    assert_eq!(a.checked_mul(b), Some(Fraction::new(1, 1)));

    // 2. Addition where intermediate numerator and denominator exceed u64::MAX (18_446_744_073_709_551_615):
    //    f1 = i64::MAX / (u64::MAX / 2), f2 = i64::MAX / (u64::MAX / 2)
    //    lhs and rhs in checked_add = i64::MAX * (u64::MAX / 2) = 8.507059e36 (> u64::MAX).
    //    Intermediate numer = 1.7014118e37 (> u64::MAX).
    //    Intermediate denom = (u64::MAX / 2)^2 = 8.507059e36 (> u64::MAX).
    //    GCD reduction reduces it to 2/1.
    let half_u64_max = NonZeroU64::new(u64::MAX / 2).unwrap();
    let f1 = Fraction::new_nonzero(i64::MAX, half_u64_max);
    let f2 = Fraction::new_nonzero(i64::MAX, half_u64_max);
    assert_eq!(f1 + f2, Fraction::new(2, 1));
    assert_eq!(f1.checked_add(f2), Some(Fraction::new(2, 1)));
}

/// Checked operations return `None` when reduced results exceed 64-bit representation
/// or when intermediate values exceed 128-bit limits.
#[test]
fn test_intermediate_128bit_overflow_checked() {
    // 1. Numerator overflow (> i64::MAX)
    let max_i64 = Fraction::from_integer(i64::MAX);
    let min_i64 = Fraction::from_integer(i64::MIN);
    let two = Fraction::from_integer(2);
    let one = Fraction::from_integer(1);

    assert_eq!(max_i64.checked_mul(two), None);
    assert_eq!(min_i64.checked_mul(two), None);
    assert_eq!(max_i64.checked_add(one), None);
    assert_eq!(min_i64.checked_sub(one), None);

    // 2. Denominator overflow (> u64::MAX)
    let max_u64_denom = Fraction::new_nonzero(1, NonZeroU64::new(u64::MAX).unwrap());
    let half = Fraction::new(1, 2);
    // Intermediate denom = u64::MAX * 2 (exceeds u64::MAX, fits in u128, but GCD cannot reduce below u64::MAX)
    assert_eq!(max_u64_denom.checked_mul(half), None);

    // 3. Intermediate 128-bit addition overflow (> u128::MAX)
    //    lhs_magnitude = i64::MAX * u64::MAX ≈ 1.7014e38
    //    rhs_magnitude = i64::MAX * u64::MAX ≈ 1.7014e38
    //    lhs + rhs ≈ 3.4028e38 > u128::MAX (3.40282366920938463463374607431768211455e38)
    let big_f1 = Fraction::new_nonzero(i64::MAX, NonZeroU64::new(u64::MAX).unwrap());
    let big_f2 = Fraction::new_nonzero(i64::MAX, NonZeroU64::new(u64::MAX).unwrap());
    assert_eq!(big_f1.checked_add(big_f2), None);
}

/// Standard operator overloads (+, -, *, /, neg) panic when results exceed 64-bit limits.
#[test]
#[should_panic(expected = "fraction multiplication overflowed its representation")]
fn test_panic_on_mul_overflow() {
    let _ = Fraction::from_integer(i64::MAX) * Fraction::from_integer(2);
}

#[test]
#[should_panic(expected = "fraction addition overflowed its representation")]
fn test_panic_on_add_overflow() {
    let _ = Fraction::from_integer(i64::MAX) + Fraction::from_integer(1);
}

#[test]
#[should_panic(expected = "fraction subtraction overflowed its representation")]
fn test_panic_on_sub_overflow() {
    let _ = Fraction::from_integer(i64::MIN) - Fraction::from_integer(1);
}

#[test]
#[should_panic(expected = "fraction division by zero or overflow")]
fn test_panic_on_div_overflow() {
    let max_u64_denom = Fraction::new_nonzero(1, NonZeroU64::new(u64::MAX).unwrap());
    let two = Fraction::from_integer(2);
    let _ = max_u64_denom / two;
}

#[test]
#[should_panic(expected = "fraction negation overflowed its numerator")]
fn test_panic_on_neg_overflow() {
    let _ = -Fraction::from_integer(i64::MIN);
}


