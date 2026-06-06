//! Hashing primitives for akari.
//!
//! - [`FxHasher`] / [`HashMap`] — general-purpose. Std builds use
//!   `std::collections::HashMap` with `RandomState`; no_std builds use
//!   `hashbrown::HashMap` keyed by [`FxHasher`].
//! - [`IdHasher`] / `IdHashMap*` aliases — identity hasher for keys that
//!   are already hashes (e.g. [`TypeId`](core::any::TypeId)) or caller-
//!   controlled integers.
//!
//! Internal akari code MUST import `HashMap` from this module so the
//! std / no_std switch stays transparent.
//!
//! # Security
//!
//! Neither [`FxHasher`] (under no_std) nor [`IdHasher`] is DoS-resistant.
//! Don't route attacker-controlled keys through them without upstream
//! rate-limiting.

use core::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
const ROTATE: u32 = 5;

/// Fast non-cryptographic hasher, FxHash-style.
#[derive(Default, Clone)]
pub struct FxHasher {
    hash: u64,
}

impl Hasher for FxHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }

    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        while bytes.len() >= 8 {
            let word = u64::from_ne_bytes(bytes[..8].try_into().unwrap());
            self.hash = (self.hash.rotate_left(ROTATE) ^ word).wrapping_mul(SEED);
            bytes = &bytes[8..];
        }
        if bytes.len() >= 4 {
            let word = u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as u64;
            self.hash = (self.hash.rotate_left(ROTATE) ^ word).wrapping_mul(SEED);
            bytes = &bytes[4..];
        }
        for &b in bytes {
            self.hash = (self.hash.rotate_left(ROTATE) ^ b as u64).wrapping_mul(SEED);
        }
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.hash = (self.hash.rotate_left(ROTATE) ^ n).wrapping_mul(SEED);
    }
}

/// `BuildHasher` for [`FxHasher`].
pub type FxBuildHasher = BuildHasherDefault<FxHasher>;

#[cfg(not(feature = "no_std"))]
pub use std::collections::HashMap;

#[cfg(feature = "no_std")]
pub type HashMap<K, V> = hashbrown::HashMap<K, V, FxBuildHasher>;

/// Identity hasher backed by `u128` storage, so every typed `write_*`
/// method is a single widening assignment.
///
/// Pair with [`IdBuildHasher`] and one of the `IdHashMap*` aliases below.
///
/// # When to use
///
/// - [`TypeId`](core::any::TypeId) keys (uniform across the full width).
/// - Unsigned integer keys.
/// - Signed keys known to stay non-negative.
///
/// Avoid wide-range signed keys — see [`IdHasher::finish`] for the reason.
#[derive(Default, Clone, Debug)]
pub struct IdHasher(u128);

impl Hasher for IdHasher {
    /// XOR-folds the `u128` storage into a `u64`, then byte-reverses.
    ///
    /// The reverse moves the value's low-bit entropy into the high bits
    /// where hashbrown extracts its SIMD control byte. Without it,
    /// identity-hashing small ints degrades sharply on large tables —
    /// every entry's control byte would end up zero.
    ///
    /// # Signed-key collisions
    ///
    /// The XOR-fold treats sign extension as noise: for any signed key
    /// `n` shorter than 128 bits, `n` and `!n` produce the same hash —
    /// `i32::MAX` collides with `i32::MIN`, `i64::MAX` with `i64::MIN`,
    /// and so on.
    #[inline]
    fn finish(&self) -> u64 {
        ((self.0 as u64) ^ ((self.0 >> 64) as u64)).swap_bytes()
    }

    #[inline] fn write_u8   (&mut self, n: u8)    { self.0 = n as u128; }
    #[inline] fn write_u16  (&mut self, n: u16)   { self.0 = n as u128; }
    #[inline] fn write_u32  (&mut self, n: u32)   { self.0 = n as u128; }
    #[inline] fn write_u64  (&mut self, n: u64)   { self.0 = n as u128; }
    #[inline] fn write_u128 (&mut self, n: u128)  { self.0 = n; }
    #[inline] fn write_usize(&mut self, n: usize) { self.0 = n as u128; }

    #[inline] fn write_i8   (&mut self, n: i8)    { self.0 = (n as i128) as u128; }
    #[inline] fn write_i16  (&mut self, n: i16)   { self.0 = (n as i128) as u128; }
    #[inline] fn write_i32  (&mut self, n: i32)   { self.0 = (n as i128) as u128; }
    #[inline] fn write_i64  (&mut self, n: i64)   { self.0 = (n as i128) as u128; }
    #[inline] fn write_i128 (&mut self, n: i128)  { self.0 = n as u128; }
    #[inline] fn write_isize(&mut self, n: isize) { self.0 = (n as i128) as u128; }

    /// Defensive byte-fold for `Hash` impls that bypass the typed
    /// `write_*` methods. Well-defined but not a good distribution; the
    /// `TypeId` and integer keys this hasher targets hit the typed fast
    /// paths.
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.rotate_left(5) ^ (b as u128);
        }
    }
}

/// `BuildHasher` for [`IdHasher`].
pub type IdBuildHasher = BuildHasherDefault<IdHasher>;

// One alias per integer width plus a `TypeId` alias. The std and no_std
// blocks differ only in which `HashMap` they import.

#[cfg(not(feature = "no_std"))]
mod id_map_aliases {
    use super::IdBuildHasher;
    use core::any::TypeId;
    use std::collections::HashMap;
    pub type IdHashMapU8    <V> = HashMap<u8,     V, IdBuildHasher>;
    pub type IdHashMapU16   <V> = HashMap<u16,    V, IdBuildHasher>;
    pub type IdHashMapU32   <V> = HashMap<u32,    V, IdBuildHasher>;
    pub type IdHashMapU64   <V> = HashMap<u64,    V, IdBuildHasher>;
    pub type IdHashMapU128  <V> = HashMap<u128,   V, IdBuildHasher>;
    pub type IdHashMapI8    <V> = HashMap<i8,     V, IdBuildHasher>;
    pub type IdHashMapI16   <V> = HashMap<i16,    V, IdBuildHasher>;
    pub type IdHashMapI32   <V> = HashMap<i32,    V, IdBuildHasher>;
    pub type IdHashMapI64   <V> = HashMap<i64,    V, IdBuildHasher>;
    pub type IdHashMapI128  <V> = HashMap<i128,   V, IdBuildHasher>;
    pub type IdHashMapUsize <V> = HashMap<usize,  V, IdBuildHasher>;
    pub type IdHashMapIsize <V> = HashMap<isize,  V, IdBuildHasher>;
    pub type IdHashMapTypeId<V> = HashMap<TypeId, V, IdBuildHasher>;
}

#[cfg(feature = "no_std")]
mod id_map_aliases {
    use super::IdBuildHasher;
    use core::any::TypeId;
    use hashbrown::HashMap;
    pub type IdHashMapU8    <V> = HashMap<u8,     V, IdBuildHasher>;
    pub type IdHashMapU16   <V> = HashMap<u16,    V, IdBuildHasher>;
    pub type IdHashMapU32   <V> = HashMap<u32,    V, IdBuildHasher>;
    pub type IdHashMapU64   <V> = HashMap<u64,    V, IdBuildHasher>;
    pub type IdHashMapU128  <V> = HashMap<u128,   V, IdBuildHasher>;
    pub type IdHashMapI8    <V> = HashMap<i8,     V, IdBuildHasher>;
    pub type IdHashMapI16   <V> = HashMap<i16,    V, IdBuildHasher>;
    pub type IdHashMapI32   <V> = HashMap<i32,    V, IdBuildHasher>;
    pub type IdHashMapI64   <V> = HashMap<i64,    V, IdBuildHasher>;
    pub type IdHashMapI128  <V> = HashMap<i128,   V, IdBuildHasher>;
    pub type IdHashMapUsize <V> = HashMap<usize,  V, IdBuildHasher>;
    pub type IdHashMapIsize <V> = HashMap<isize,  V, IdBuildHasher>;
    pub type IdHashMapTypeId<V> = HashMap<TypeId, V, IdBuildHasher>;
}

pub use id_map_aliases::*;
