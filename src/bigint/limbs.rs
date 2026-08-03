//! # Limb representation and normalization
//!
//! [`BigUint`] is an arbitrary-precision **unsigned** integer stored as a
//! `Vec<u64>` of *limbs* in **little-endian limb order**: `limbs[0]` holds the
//! least significant 64 bits, `limbs[1]` the next 64, and so on. The numeric
//! value is therefore
//!
//! ```text
//! value = sum over i of limbs[i] * 2^(64 * i)
//! ```
//!
//! ## Why little-endian limbs
//!
//! Every carry-propagating loop (add, sub, mul, shift) walks from the least
//! significant limb upward, so index 0 being the least significant limb means
//! the loops read `0..n` in memory order and growth is a `push` at the end of
//! the vector rather than an insert at the front. Big-endian *bytes* are still
//! the external interface (see [`BigUint::from_be_bytes`] and
//! [`BigUint::to_be_bytes`]) because RSA's I2OSP/OS2IP are defined that way.
//!
//! ## Normalization invariant (load-bearing)
//!
//! A `BigUint` is **normalized**: it never stores trailing zero limbs, so the
//! canonical representation of zero is the empty vector. Every constructor in
//! this crate funnels through one internal `normalized` helper, which trims.
//!
//! This is not cosmetic. Two things break without it:
//!
//! - **Equality.** `PartialEq` is derived, so it compares the limb vectors
//!   directly. An unnormalized `[5, 0]` would *not* equal the normalized `[5]`
//!   even though both mean 5.
//! - **[`BigUint::bit_len`]**, which reads `leading_zeros` of the top limb and
//!   assumes that limb is nonzero. With a trailing zero limb it would report 64
//!   bits too many (or a nonsense length for `[0]`).
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! let value = BigUint::from_limbs_le(vec![7, 0, 0]);
//! assert_eq!(value.limbs(), &[7]);
//! assert_eq!(value, BigUint::from_u64(7));
//! assert!(BigUint::from_limbs_le(vec![0, 0]).is_zero());
//! ```

/// An arbitrary-precision unsigned integer over little-endian `u64` limbs.
///
/// See the [module docs](self) for the representation and the normalization
/// invariant. Construct with [`BigUint::zero`], [`BigUint::from_u64`],
/// [`BigUint::from_limbs_le`], [`BigUint::from_be_bytes`], or
/// [`BigUint::from_hex`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::bigint::BigUint;
///
/// let a = BigUint::from_u64(1_000);
/// let b = BigUint::from_u64(24);
/// assert_eq!(a.add(&b), BigUint::from_u64(1_024));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BigUint {
    /// Little-endian limbs with no trailing zeros. `pub(crate)` so the sibling
    /// arithmetic modules can read it; external code uses [`BigUint::limbs`].
    pub(crate) limbs: Vec<u64>,
}

impl BigUint {
    /// The value zero, represented by an empty limb vector.
    ///
    /// # Returns
    ///
    /// A normalized `BigUint` equal to 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert!(BigUint::zero().is_zero());
    /// assert_eq!(BigUint::zero().limbs().len(), 0);
    /// ```
    pub const fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    /// Build a value from a single 64-bit integer.
    ///
    /// # Arguments
    ///
    /// * `value` — the numeric value; `0` yields the canonical empty vector.
    ///
    /// # Returns
    ///
    /// A normalized `BigUint` equal to `value`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_u64(u64::MAX).bit_len(), 64);
    /// assert_eq!(BigUint::from_u64(0), BigUint::zero());
    /// ```
    pub fn from_u64(value: u64) -> Self {
        Self::normalized(vec![value])
    }

    /// Build a value from raw little-endian limbs, trimming trailing zeros.
    ///
    /// # Arguments
    ///
    /// * `limbs` — limbs with `limbs[0]` least significant. Trailing zeros are
    ///   permitted here and removed; see the [module docs](self).
    ///
    /// # Returns
    ///
    /// The normalized `BigUint`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// // 2^64 + 1
    /// let value = BigUint::from_limbs_le(vec![1, 1]);
    /// assert_eq!(value.bit_len(), 65);
    /// ```
    pub fn from_limbs_le(limbs: Vec<u64>) -> Self {
        Self::normalized(limbs)
    }

    /// Borrow the normalized little-endian limbs.
    ///
    /// # Returns
    ///
    /// A slice with the least significant limb first and no trailing zeros, so
    /// an empty slice means zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert_eq!(BigUint::from_u64(5).limbs(), &[5]);
    /// ```
    pub fn limbs(&self) -> &[u64] {
        &self.limbs
    }

    /// Test for zero.
    ///
    /// # Returns
    ///
    /// `true` when the value is 0. Thanks to normalization this is a length
    /// check, not a scan.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert!(BigUint::zero().is_zero());
    /// assert!(!BigUint::from_u64(1).is_zero());
    /// ```
    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    /// Test for one, which the modular paths special-case.
    ///
    /// # Returns
    ///
    /// `true` when the value is exactly 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// assert!(BigUint::from_u64(1).is_one());
    /// assert!(!BigUint::zero().is_one());
    /// ```
    pub fn is_one(&self) -> bool {
        self.limbs.len() == 1 && self.limbs[0] == 1
    }

    /// Wrap limbs into a normalized value. The single internal constructor.
    pub(crate) fn normalized(mut limbs: Vec<u64>) -> Self {
        trim(&mut limbs);
        Self { limbs }
    }
}

/// Drop trailing zero limbs in place, restoring the normalization invariant.
pub(crate) fn trim(limbs: &mut Vec<u64>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}
