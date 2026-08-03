//! Decimal and hexadecimal rendering for [`Uint`].
//!
//! Decimal output divides by `10^19` — the largest power of ten below `2^64` —
//! so each single-limb division yields nineteen digits at once instead of one.
//! Every chunk but the most significant is zero-padded to nineteen digits;
//! forgetting that padding is the classic bug here (`1_000000000000000000`
//! would print as `11`).

use super::uint::Uint;

/// The largest power of ten that fits in a `u64`.
const CHUNK: u64 = 10_000_000_000_000_000_000;

impl Uint {
    /// Renders the value in decimal, without a sign or separators.
    ///
    /// # Returns
    ///
    /// The decimal digits; `"0"` for zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::zero().to_dec_string(), "0");
    /// assert_eq!(Uint::from_limbs(vec![0, 1]).to_dec_string(), "18446744073709551616");
    /// let big = Uint::from_dec_str("123456789012345678901234567890").unwrap();
    /// assert_eq!(big.to_dec_string(), "123456789012345678901234567890");
    /// ```
    pub fn to_dec_string(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut chunks: Vec<u64> = Vec::new();
        let mut rest = self.clone();
        while !rest.is_zero() {
            let (q, r) = rest.divmod_u64(CHUNK);
            chunks.push(r);
            rest = q;
        }
        let mut out = chunks.pop().unwrap_or(0).to_string();
        for chunk in chunks.iter().rev() {
            out.push_str(&format!("{chunk:019}"));
        }
        out
    }

    /// Renders the value in lowercase hexadecimal with no `0x` prefix.
    ///
    /// # Returns
    ///
    /// The minimal hex digits; `"0"` for zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::zero().to_hex_string(), "0");
    /// assert_eq!(Uint::from_u64(255).to_hex_string(), "ff");
    /// assert_eq!(Uint::from_limbs(vec![0, 1]).to_hex_string(), "10000000000000000");
    /// ```
    pub fn to_hex_string(&self) -> String {
        let Some((&top, rest)) = self.limbs.split_last() else {
            return "0".to_string();
        };
        let mut out = format!("{top:x}");
        for limb in rest.iter().rev() {
            out.push_str(&format!("{limb:016x}"));
        }
        out
    }
}
