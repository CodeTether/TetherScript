//! Ordering for [`Uint`].
//!
//! Because limbs are normalized (no trailing zeros), comparison is simply
//! "shorter is smaller", then a most-significant-first limb scan. Skipping
//! normalization would break this: `[5, 0]` and `[5]` are the same number but
//! have different lengths.
//!
//! `Ord`/`PartialOrd` are implemented here rather than derived, since the
//! derived lexicographic order on a little-endian `Vec<u64>` would be wrong.

use std::cmp::Ordering;

use super::uint::Uint;

impl Uint {
    /// Compares two values numerically.
    ///
    /// # Arguments
    ///
    /// * `other` — the right-hand side.
    ///
    /// # Returns
    ///
    /// [`Ordering::Less`], [`Ordering::Equal`], or [`Ordering::Greater`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use tetherscript::bignum::Uint;
    ///
    /// let small = Uint::from_u64(u64::MAX);
    /// let big = Uint::from_limbs(vec![0, 1]);
    /// assert_eq!(small.cmp_uint(&big), Ordering::Less);
    /// assert_eq!(big.cmp_uint(&big), Ordering::Equal);
    /// assert!(Uint::zero() < Uint::one());
    /// ```
    pub fn cmp_uint(&self, other: &Uint) -> Ordering {
        if self.limbs.len() != other.limbs.len() {
            return self.limbs.len().cmp(&other.limbs.len());
        }
        // Equal lengths: scan most significant limb first.
        for (a, b) in self.limbs.iter().zip(&other.limbs).rev() {
            let ord = a.cmp(b);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    }
}

impl Ord for Uint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_uint(other)
    }
}

impl PartialOrd for Uint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Delegate to Ord, as clippy's non_canonical_partial_ord_impl requires.
        Some(self.cmp(other))
    }
}
