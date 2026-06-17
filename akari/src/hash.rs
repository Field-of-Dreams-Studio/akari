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

/// Identity hasher backed by `u128` storage.
///
/// Each typed `write_*` method is responsible for spreading its input
/// across the full 64-bit hash space at *write time*, where the input
/// width is known. [`finish`](IdHasher::finish) then just truncates the
/// storage to `u64` — no further mixing.
///
/// Pair with [`IdBuildHasher`] and one of the `IdHashMap*` aliases below.
///
/// # Why bit-spreading lives in `write_*`, not `finish`
///
/// `Hasher::finish` is type-erased — it doesn't know whether the input
/// was a `u8` or a `u128`. Hashbrown needs entropy across the full
/// `u64` return: low bits select the bucket index, high seven bits
/// select the SIMD control byte. A naive truncation `self.0 as u64`
/// leaves the high half of the output zero for any narrow input
/// (u8/u16/u32), causing every entry's control byte to collapse to 0
/// and the SIMD probe to degenerate into a linear scan. Putting the
/// spreading at the typed `write_*` boundary is the cleanest fix:
/// each method knows its own width and applies the right broadcast.
///
/// # Injectivity
///
/// `write_u8`, `write_u16`, `write_u32`, and `write_u64` are
/// **injective** — distinct inputs produce distinct stored values:
///
/// - `write_u8(n)` stores `n × 0x0101_0101_0101_0101`, broadcasting the
///   byte into all 8 lanes without inter-lane carries (since `n < 256`).
///   Byte 0 of the result equals `n` exactly.
/// - `write_u16(n)` stores `n × 0x0001_0001_0001_0001`, broadcasting
///   into all 4 u16 lanes without carries (since `n < 2¹⁶`). Bits
///   `[0..16)` of the result equal `n` exactly.
/// - `write_u32(n)` stores `(n as u64) | ((n as u64) << 32)`, placing
///   `n` in both halves of the u64. Bits `[0..32)` equal `n` exactly.
/// - `write_u64(n)` stores `n` unchanged.
///
/// `write_u128` is **necessarily lossy** (u128 → u64 has 2⁶⁴ × the
/// codomain size of the domain). The implementation XOR-folds the two
/// u64 halves, a balanced compression where each output value has
/// exactly 2⁶⁴ preimages distributed uniformly. For `TypeId` and
/// other entropy-spanning u128 inputs the collision rate is the
/// birthday-baseline 2⁻⁶⁴ per pair; for adversary-chosen u128 keys
/// collisions are trivially constructable (`hi = lo ⊕ target`), which
/// is why `IdHasher` is *not* DoS-resistant by design.
///
/// Signed variants `write_i*` delegate to their unsigned counterparts
/// via bitwise `as`-cast, preserving the bit pattern.
#[derive(Default, Clone, Debug)]
pub struct IdHasher(u128);

impl Hasher for IdHasher {
    /// Truncate the `u128` storage to `u64`. All entropy distribution
    /// has already been performed by the typed `write_*` method that
    /// produced the storage value — see the type-level docstring.
    #[inline]
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    // ----- Unsigned widths: each spreads its input across full u64. -----

    #[inline]
    fn write_u8(&mut self, n: u8) {
        // Broadcast n into all 8 byte lanes: 0x_NN_NN_NN_NN_NN_NN_NN_NN.
        self.0 = ((n as u64).wrapping_mul(0x0101_0101_0101_0101)) as u128;
    }

    #[inline]
    fn write_u16(&mut self, n: u16) {
        // Broadcast n into all 4 u16 lanes: 0x_NNNN_NNNN_NNNN_NNNN.
        self.0 = ((n as u64).wrapping_mul(0x0001_0001_0001_0001)) as u128;
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        // Duplicate n into both halves: 0x_NNNN_NNNN_NNNN_NNNN.
        let x = n as u64;
        self.0 = ((x << 32) | x) as u128;
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        // n already has entropy across all 64 bits; store as-is.
        self.0 = n as u128;
    }

    #[inline]
    fn write_u128(&mut self, n: u128) {
        // Necessarily lossy: XOR-fold the two halves into u64.
        let hi = (n >> 64) as u64;
        let lo = n as u64;
        self.0 = (hi ^ lo) as u128;
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        // Use the matching-width unsigned spread for the target platform.
        #[cfg(target_pointer_width = "64")]
        self.write_u64(n as u64);
        #[cfg(target_pointer_width = "32")]
        self.write_u32(n as u32);
        #[cfg(target_pointer_width = "16")]
        self.write_u16(n as u16);
    }

    // ----- Signed widths: bit-cast to unsigned, then spread. -----

    #[inline] fn write_i8   (&mut self, n: i8)    { self.write_u8(n as u8);     }
    #[inline] fn write_i16  (&mut self, n: i16)   { self.write_u16(n as u16);   }
    #[inline] fn write_i32  (&mut self, n: i32)   { self.write_u32(n as u32);   }
    #[inline] fn write_i64  (&mut self, n: i64)   { self.write_u64(n as u64);   }
    #[inline] fn write_i128 (&mut self, n: i128)  { self.write_u128(n as u128); }
    #[inline] fn write_isize(&mut self, n: isize) {
        #[cfg(target_pointer_width = "64")]
        self.write_u64(n as u64);
        #[cfg(target_pointer_width = "32")]
        self.write_u32(n as u32);
        #[cfg(target_pointer_width = "16")]
        self.write_u16(n as u16);
    }

    /// Defensive byte-fold for `Hash` impls that bypass the typed
    /// `write_*` methods. Well-defined but not a good distribution;
    /// the `TypeId` and integer keys this hasher targets hit the typed
    /// fast paths and never reach this branch.
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
