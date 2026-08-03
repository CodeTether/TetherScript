//! Bit packing for base64url decoding.
//!
//! Split from `super::jwks_base64url` so that file owns alphabet validation
//! only and this one owns the 6-bit-to-8-bit regrouping. Neither concern needs
//! to know about the other beyond the sextet slice they exchange.

/// Regroup validated sextets into bytes.
///
/// # Arguments
///
/// * `sextets` — 6-bit values, already confirmed by the caller to be in the
///   base64url alphabet and to form a decodable length.
///
/// # Returns
///
/// The decoded bytes. A trailing group of 2 sextets yields 1 byte and a group
/// of 3 yields 2 bytes, which is how unpadded base64url encodes a length that
/// is not a multiple of three.
///
/// # Errors
///
/// Cannot fail: validation is the caller's job, so this function is total.
///
/// # Examples
///
/// ```tether
/// // Reached from script code through `jwks_parse`; never called directly.
/// println(str(jwks_parse("{\"keys\":[]}").is_ok()))   // true
/// ```
pub(super) fn pack(sextets: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(sextets.len() / 4 * 3);
    for block in sextets.chunks(4) {
        let a = block[0];
        let b = block.get(1).copied().unwrap_or(0);
        let c = block.get(2).copied().unwrap_or(0);
        let d = block.get(3).copied().unwrap_or(0);
        out.push((a << 2) | (b >> 4));
        if block.len() > 2 {
            out.push((b << 4) | (c >> 2));
        }
        if block.len() > 3 {
            out.push((c << 6) | d);
        }
    }
    out
}
