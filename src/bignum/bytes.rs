//! Big-endian byte conversion for [`Uint`].
//!
//! Big-endian ("network order") is what every cryptographic format uses —
//! RSA moduli, signatures, and JWK `n`/`e` values are all big-endian — so it is
//! the only byte order offered, to remove any chance of a silent mix-up.
//!
//! Two output forms are provided: [`Uint::to_be_bytes`] emits the minimal
//! encoding (empty for zero), and [`Uint::to_be_bytes_padded`] emits a fixed
//! width, which is what a signature comparison needs.

use super::uint::Uint;

impl Uint {
    /// Parses a big-endian byte string.
    ///
    /// # Arguments
    ///
    /// * `bytes` — big-endian magnitude, most significant byte first. Leading
    ///   zero bytes are accepted and normalized away. An empty slice is zero.
    ///
    /// # Returns
    ///
    /// The value the bytes encode.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::from_be_bytes(&[]), Uint::zero());
    /// assert_eq!(Uint::from_be_bytes(&[0, 0, 1]), Uint::one());
    /// assert_eq!(Uint::from_be_bytes(&[1, 0]), Uint::from_u64(256));
    /// ```
    pub fn from_be_bytes(bytes: &[u8]) -> Uint {
        let mut limbs = Vec::with_capacity(bytes.len() / 8 + 1);
        // Walk from the least significant end in 8-byte groups.
        let mut rest = bytes;
        while !rest.is_empty() {
            let split = rest.len().saturating_sub(8);
            let (head, tail) = rest.split_at(split);
            let mut limb: u64 = 0;
            for &byte in tail {
                limb = (limb << 8) | byte as u64;
            }
            limbs.push(limb);
            rest = head;
        }
        Uint::from_limbs(limbs)
    }

    /// Serializes to the minimal big-endian encoding.
    ///
    /// # Returns
    ///
    /// The magnitude with no leading zero bytes; an **empty** vector for zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert!(Uint::zero().to_be_bytes().is_empty());
    /// assert_eq!(Uint::from_u64(256).to_be_bytes(), vec![1, 0]);
    /// ```
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let width = self.bit_len().div_ceil(8);
        self.to_be_bytes_padded(width)
    }

    /// Serializes to exactly `width` big-endian bytes, zero-padded on the left.
    ///
    /// # Arguments
    ///
    /// * `width` — the exact output length in bytes.
    ///
    /// # Panics
    ///
    /// Panics when the value does not fit in `width` bytes, naming both sizes;
    /// truncating a modulus silently would be far worse.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::bignum::Uint;
    ///
    /// assert_eq!(Uint::one().to_be_bytes_padded(4), vec![0, 0, 0, 1]);
    /// assert_eq!(Uint::zero().to_be_bytes_padded(2), vec![0, 0]);
    /// ```
    pub fn to_be_bytes_padded(&self, width: usize) -> Vec<u8> {
        let needed = self.bit_len().div_ceil(8);
        assert!(
            needed <= width,
            "Uint needs {needed} bytes but only {width} were requested"
        );
        let mut out = vec![0u8; width];
        // Byte `i` from the right is byte `i % 8` of limb `i / 8`.
        for (i, slot) in out.iter_mut().rev().enumerate() {
            *slot = (self.limb(i / 8) >> (8 * (i % 8))) as u8;
        }
        out
    }
}
