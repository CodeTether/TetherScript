//! The [`Uint`] arbitrary-precision unsigned integer: representation and core
//! predicates.
//!
//! # Representation
//!
//! A `Uint` is a `Vec<u64>` of **little-endian limbs**: `limbs[0]` is the least
//! significant 64 bits. The vector is always *normalized* — it never ends in a
//! zero limb — so zero is the empty vector and every value has exactly one
//! representation. That invariant is what makes derived `PartialEq`/`Hash`,
//! [`Uint::cmp_uint`], and [`Uint::bit_len`] correct; construct values only
//! through [`Uint::from_limbs`] (or the higher-level constructors), never by
//! filling `limbs` directly.
//!
//! # Module wiring
//!
//! This file is one concern of the `bignum` module. The integrator's
//! `src/bignum/mod.rs` is expected to read roughly:
//!
//! ```text
//! pub mod uint;
//! mod add;
//! mod bits;
//! mod bytes;
//! mod cmp;
//! mod display;
//! mod div;
//! mod div_estimate;
//! mod div_knuth;
//! mod div_mulsub;
//! mod div_step;
//! mod error;
//! mod format;
//! mod modpow;
//! mod modular;
//! mod mul;
//! mod parse;
//! mod shift;
//! mod sub;
//!
//! pub use error::ParseUintError;
//! pub use uint::Uint;
//! ```
//!
//! # Examples
//!
//! ```
//! use tetherscript::bignum::Uint;
//!
//! let zero = Uint::zero();
//! assert!(zero.is_zero());
//! assert_eq!(zero.limbs(), &[] as &[u64]);
//!
//! let one = Uint::one();
//! assert!(one.is_one());
//! assert!(one.is_odd());
//! assert_eq!(one.limbs(), &[1]);
//! ```

/// An arbitrary-precision unsigned integer stored as little-endian `u64` limbs.
///
/// The limb vector is always normalized (no trailing zero limbs), so `Uint`
/// values compare and hash structurally. Zero is the empty limb vector.
///
/// # Examples
///
/// ```
/// use tetherscript::bignum::Uint;
///
/// let a = Uint::from_u64(7);
/// let b = Uint::from_u64(35);
/// assert_eq!(a.mul(&b), Uint::from_u64(245));
/// assert_eq!(b.rem(&a), Uint::zero());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Uint {
    /// Little-endian limbs with no trailing zero limb.
    pub(crate) limbs: Vec<u64>,
}

impl Uint {
    /// Returns zero.
    ///
    /// # Returns
    ///
    /// The value `0`, represented by an empty limb vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert!(Uint::zero().is_zero());
    /// ```
    pub const fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    /// Returns one.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::one(), Uint::from_u64(1));
    /// ```
    pub fn one() -> Self {
        Self::from_u64(1)
    }

    /// Builds a `Uint` from a single 64-bit value.
    ///
    /// # Arguments
    ///
    /// * `value` — the numeric value.
    ///
    /// # Returns
    ///
    /// `value` as a `Uint`; `0` yields the normalized empty representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(0), Uint::zero());
    /// assert_eq!(Uint::from_u64(u64::MAX).bit_len(), 64);
    /// ```
    pub fn from_u64(value: u64) -> Self {
        if value == 0 {
            Self::zero()
        } else {
            Self { limbs: vec![value] }
        }
    }

    /// Builds a `Uint` from little-endian limbs, normalizing away trailing
    /// zero limbs.
    ///
    /// # Arguments
    ///
    /// * `limbs` — little-endian limbs, least significant first. Trailing zero
    ///   limbs are permitted and removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// let padded = Uint::from_limbs(vec![5, 0, 0]);
    /// assert_eq!(padded.limbs(), &[5]);
    /// assert_eq!(padded, Uint::from_u64(5));
    /// ```
    pub fn from_limbs(mut limbs: Vec<u64>) -> Self {
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
        Self { limbs }
    }

    /// Returns the normalized little-endian limbs.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(1).limbs(), &[1]);
    /// ```
    pub fn limbs(&self) -> &[u64] {
        &self.limbs
    }

    /// Returns limb `index`, or zero when the index is past the top limb.
    ///
    /// Internal helper: it lets the arithmetic loops treat every value as
    /// infinitely zero-extended instead of special-casing lengths.
    pub(crate) fn limb(&self, index: usize) -> u64 {
        self.limbs.get(index).copied().unwrap_or(0)
    }

    /// Returns `true` when the value is zero.
    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    /// Returns `true` when the value is exactly one.
    ///
    /// Used by [`Uint::mod_pow`], where a modulus of one is a special case.
    pub fn is_one(&self) -> bool {
        self.limbs == [1]
    }

    /// Returns `true` when the value is odd.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert!(Uint::from_u64(3).is_odd());
    /// assert!(!Uint::zero().is_odd());
    /// ```
    pub fn is_odd(&self) -> bool {
        (self.limb(0) & 1) == 1
    }

    /// Converts to `u64` when the value fits.
    ///
    /// # Returns
    ///
    /// `Some(value)` when the value needs at most 64 bits, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    /// assert_eq!(Uint::from_u64(9).to_u64(), Some(9));
    /// assert_eq!(Uint::from_limbs(vec![0, 1]).to_u64(), None);
    /// ```
    pub fn to_u64(&self) -> Option<u64> {
        match self.limbs.len() {
            0 => Some(0),
            1 => Some(self.limbs[0]),
            _ => None,
        }
    }
}
