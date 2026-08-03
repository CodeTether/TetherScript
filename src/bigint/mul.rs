//! # Schoolbook multiplication
//!
//! Operand-scanning (a.k.a. schoolbook, O(n·m)) multiplication over the
//! little-endian limbs described in the `limbs` module. No Karatsuba: at RSA-2048
//! sizes the operands are ~32 limbs, where the crossover has not been reached
//! and the simpler loop is the one that can be audited by reading it.
//!
//! ## Why `u128` intermediates are mandatory
//!
//! Each inner step computes `a[i] * b[j] + out[i + j] + carry`. With every term
//! at its maximum of `2^64 - 1` that is
//! `(2^64 - 1)^2 + 2·(2^64 - 1) = 2^128 - 1`, which exactly fits `u128` and
//! would overflow `u64` by a factor of `2^64`. The low 64 bits become the
//! output limb and the high 64 bits become the carry into the next `j`.
//!
//! ## Why `out[i + n] = carry` is a store, not an add
//!
//! Position `i + n` (where `n = b.len()`) is untouched when the inner loop for
//! row `i` finishes: an earlier row `i' < i` writes `i' + j` for `j < n`, and
//! `i' + j = i + n` would need `j = i + n - i' > n`. So the running carry out of
//! row `i` is the first thing ever written there, and storing it cannot lose a
//! previously accumulated value.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::bigint::BigUint;
//!
//! // 2^64 * 2^64 == 2^128
//! let two_64 = BigUint::from_limbs_le(vec![0, 1]);
//! assert_eq!(two_64.mul(&two_64).limbs(), &[0, 0, 1]);
//!
//! // Anything times zero is the canonical zero, not a zero-filled vector.
//! assert!(two_64.mul(&BigUint::zero()).is_zero());
//! ```

use super::limbs::BigUint;

impl BigUint {
    /// Multiply two values.
    ///
    /// # Arguments
    ///
    /// * `other` — the multiplicand.
    ///
    /// # Returns
    ///
    /// A normalized `self * other`, at most `self.limbs().len() +
    /// other.limbs().len()` limbs wide. Multiplication never fails; the result
    /// grows instead of wrapping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::bigint::BigUint;
    ///
    /// let a = BigUint::from_u64(12_345_678_901_234_567_890);
    /// let b = BigUint::from_limbs_le(vec![6_531_711_741_328_785_130, 5]);
    /// // 12345678901234567890 * 98765432109876543210
    /// assert_eq!(
    ///     a.mul(&b).limbs(),
    ///     &[1_331_246_629_686_034_420, 10_759_579_566_687_691_682, 3]
    /// );
    /// ```
    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let (rows, cols) = (&self.limbs, &other.limbs);
        let mut out = vec![0u64; rows.len() + cols.len()];
        for (i, &row) in rows.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &col) in cols.iter().enumerate() {
                let step = row as u128 * col as u128 + out[i + j] as u128 + carry;
                out[i + j] = step as u64;
                carry = step >> 64;
            }
            out[i + cols.len()] = carry as u64;
        }
        Self::normalized(out)
    }
}
