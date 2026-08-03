//! Significant-bit counting for big-endian integers.
//!
//! One responsibility: report how many bits a big-endian byte string actually
//! carries. Split out because both modulus sizing and error text need it, and
//! neither should own it.

/// Count significant bits in a big-endian integer.
///
/// # Arguments
///
/// * `bytes` — Big-endian integer, possibly with leading zero bytes.
///
/// # Returns
///
/// The number of bits from the most significant set bit down, so leading zero
/// bytes cannot inflate the reported strength of a key. Zero for an all-zero or
/// empty input.
///
/// # Errors
///
/// Cannot fail.
///
/// # Panics
///
/// Does not panic.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::bits::bit_length;
///
/// assert_eq!(bit_length(&[]), 0);
/// assert_eq!(bit_length(&[0x00, 0x00]), 0);
/// assert_eq!(bit_length(&[0x01]), 1);
/// assert_eq!(bit_length(&[0xff]), 8);
/// // A leading zero byte does not count toward the size.
/// assert_eq!(bit_length(&[0x00, 0xff]), 8);
/// assert_eq!(bit_length(&[0x80, 0x00]), 16);
/// ```
pub fn bit_length(bytes: &[u8]) -> usize {
    let leading = bytes.iter().take_while(|byte| **byte == 0).count();
    match bytes.get(leading) {
        None => 0,
        Some(top) => {
            let rest = (bytes.len() - leading - 1) * 8;
            rest + (8 - top.leading_zeros() as usize)
        }
    }
}
