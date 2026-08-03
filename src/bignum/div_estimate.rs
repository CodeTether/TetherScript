//! Helpers for Knuth Algorithm D: divisor normalization and the two-limb
//! quotient-digit estimate (steps D1 and D3).

/// Shifts `limbs` left by `bits` into a fixed-width buffer.
///
/// Unlike `Uint::shl` this does **not** normalize: the
/// result always has exactly `out_len` limbs, which Algorithm D relies on for
/// its indexing. Bits that would land past `out_len` are dropped; every caller
/// sizes `out_len` so that cannot happen. `bits` must be under 64, since
/// shifting a `u64` by 64 is undefined.
pub(super) fn shift_fixed(limbs: &[u64], bits: usize, out_len: usize) -> Vec<u64> {
    debug_assert!(bits < 64, "intra-limb shift must be under 64");
    let mut out = vec![0u64; out_len];
    for (i, &limb) in limbs.iter().enumerate() {
        if i < out_len {
            out[i] |= limb << bits;
        }
        if bits > 0 && i + 1 < out_len {
            out[i + 1] |= limb >> (64 - bits);
        }
    }
    out
}

/// Estimates one quotient digit from the top limbs (Algorithm D, step D3).
///
/// # Arguments
///
/// * `top` — the `n + 1` limbs of the working dividend at this position, least
///   significant first.
/// * `vn` — the normalized divisor, whose top limb has its high bit set.
///
/// # Returns
///
/// A `qhat` that is either the true digit or exactly one too large. The
/// correction loop plus normalization is what bounds the error at one; without
/// normalization the error could reach two and the single add-back in
/// `div_mulsub` would silently produce a wrong quotient.
pub(super) fn estimate(top: &[u64], vn: &[u64]) -> u64 {
    let n = vn.len();
    let base = 1u128 << 64;
    let num = ((top[n] as u128) << 64) | top[n - 1] as u128;
    let mut qhat = num / vn[n - 1] as u128;
    let mut rhat = num % vn[n - 1] as u128;
    // `||` short-circuits, so the product is only formed while qhat < 2^64;
    // both sides then stay under 2^128.
    while qhat >= base || qhat * vn[n - 2] as u128 > (rhat << 64) + top[n - 2] as u128 {
        qhat -= 1;
        rhat += vn[n - 1] as u128;
        if rhat >= base {
            break;
        }
    }
    debug_assert!(qhat < base, "normalized estimate must fit one limb");
    qhat as u64
}
