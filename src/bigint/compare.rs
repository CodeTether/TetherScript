//! # Magnitude comparison
//!
//! Comparison of two normalized [`BigUint`]s is lexicographic on limbs read
//! from the most significant end: a longer limb vector is strictly larger,
//! because normalization guarantees the top limb of the longer value is
//! nonzero. **This is why the normalization invariant is load-bearing** — an
//! unnormalized `[5, 0]` would compare as greater than `[9]` on length alone.
//!
//! [`Ord`] and [`PartialOrd`] are implemented in terms of [`BigUint::compare`],
//! so the standard operators, `max`, `min`, and sorting all agree with it.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//! use std::cmp::Ordering;
//!
//! let small = BigUint::from_u64(9);
//! let large = BigUint::from_limbs_le(vec![0, 1]); // 2^64
//! assert_eq!(small.compare(&large), Ordering::Less);
//! assert!(small < large);
//! assert_eq!(large.compare(&large), Ordering::Equal);
//! ```

use std::cmp::Ordering;

use super::limbs::BigUint;

impl BigUint {
    /// Compare this value against `other` numerically.
    ///
    /// # Arguments
    ///
    /// * `other` — the value to compare against.
    ///
    /// # Returns
    ///
    /// [`Ordering::Less`], [`Ordering::Equal`], or [`Ordering::Greater`]
    /// describing `self` relative to `other`. Runs in O(limbs) worst case and
    /// exits early on the first differing limb from the top.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    /// use std::cmp::Ordering;
    ///
    /// let a = BigUint::from_limbs_le(vec![1, 2]);
    /// let b = BigUint::from_limbs_le(vec![2, 2]);
    /// assert_eq!(a.compare(&b), Ordering::Less);
    /// assert_eq!(b.compare(&a), Ordering::Greater);
    /// assert_eq!(a.compare(&a), Ordering::Equal);
    /// ```
    pub fn compare(&self, other: &Self) -> Ordering {
        match self.limbs.len().cmp(&other.limbs.len()) {
            Ordering::Equal => {}
            unequal => return unequal,
        }
        for (mine, theirs) in self.limbs.iter().rev().zip(other.limbs.iter().rev()) {
            match mine.cmp(theirs) {
                Ordering::Equal => continue,
                unequal => return unequal,
            }
        }
        Ordering::Equal
    }
}

impl Ord for BigUint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare(other)
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
