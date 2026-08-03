//! Algorithm D steps D4–D6: multiply-and-subtract, and the rare add-back.
//!
//! `mul_sub` computes `window -= qhat * vn` over `n + 1` limbs, wrapping modulo
//! `2^(64*(n+1))` if the result would go negative and reporting that with a
//! `bool`. When it does, the estimate was exactly one too high — normalization in
//! step D1 is what guarantees "exactly one" — and `add_back` restores the value
//! by adding the divisor once. Knuth notes this happens with probability about
//! `2/2^64`, so it is nearly dead code but must still be correct.

/// Subtracts `qhat * vn` from `window` in place.
///
/// # Arguments
///
/// * `window` — `n + 1` limbs of the working dividend, least significant first.
/// * `vn` — the `n`-limb normalized divisor.
/// * `qhat` — the estimated quotient digit.
///
/// # Returns
///
/// `true` when the subtraction underflowed, meaning `qhat` was one too large.
pub(super) fn mul_sub(window: &mut [u64], vn: &[u64], qhat: u64) -> bool {
    let n = vn.len();
    let mut mul_carry: u64 = 0;
    let mut borrow: u64 = 0;
    for (slot, &v) in window[..n].iter_mut().zip(vn) {
        // (2^64-1)^2 + (2^64-1) < 2^128, so u128 holds this exactly.
        let product = qhat as u128 * v as u128 + mul_carry as u128;
        mul_carry = (product >> 64) as u64;
        borrow = take(slot, product as u64 as u128 + borrow as u128);
    }
    take(&mut window[n], mul_carry as u128 + borrow as u128) != 0
}

/// Subtracts `amount` (at most `2^64`) from `slot`, returning the borrow.
fn take(slot: &mut u64, amount: u128) -> u64 {
    let cur = *slot as u128;
    if cur >= amount {
        *slot = (cur - amount) as u64;
        0
    } else {
        *slot = (cur + (1u128 << 64) - amount) as u64;
        1
    }
}

/// Adds `vn` back into `window`, undoing one over-subtraction.
///
/// The carry out of the top limb is intentionally discarded (it wraps) because
/// it cancels the borrow that `mul_sub` reported.
pub(super) fn add_back(window: &mut [u64], vn: &[u64]) {
    let n = vn.len();
    let mut carry: u64 = 0;
    for (slot, &v) in window[..n].iter_mut().zip(vn) {
        let sum = *slot as u128 + v as u128 + carry as u128;
        *slot = sum as u64;
        carry = (sum >> 64) as u64;
    }
    window[n] = window[n].wrapping_add(carry);
}
