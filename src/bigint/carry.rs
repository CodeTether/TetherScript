//! # Carry and borrow propagation over raw limb slices
//!
//! The two primitive loops every other operation is built from. They work on
//! `&[u64]` slices in little-endian limb order (see the `limbs` module) so that
//! the `divmod` loop can reuse them on scratch buffers without allocating a
//! [`BigUint`](crate::bigint::BigUint) per step.
//!
//! ## Carry propagation
//!
//! A single limb addition can overflow 64 bits, so each step widens to `u128`:
//! `long[i] as u128 + short[i] as u128 + carry`. The low 64 bits are the output
//! limb, the high bits are the carry into the next limb. Because both addends
//! and the carry are at most `2^64 - 1`, the sum is at most `2^65 - 1` and the
//! carry out is always 0 or 1 — but it must still be *propagated*, since adding
//! 1 to `[MAX, MAX, MAX]` carries through every limb and grows the vector. This
//! is `u64::carrying_add` semantics, spelled out because that method is not
//! stable.
//!
//! ## Borrow propagation
//!
//! Subtraction mirrors it with two [`u64::overflowing_sub`] steps per limb: one
//! for the operand, one for the incoming borrow. Either can wrap, so the borrow
//! out is the OR of the two flags. The interesting case is a borrow *through* a
//! zero limb: subtracting 1 from `[0, 0, 1]` makes limb 0 wrap to `MAX` with a
//! borrow, then limb 0's borrow makes the zero limb 1 wrap to `MAX` with
//! another borrow, and only limb 2 absorbs it. A loop that forgot to OR the two
//! flags would drop that second borrow and return a wrong answer.

/// Add two little-endian limb slices.
///
/// # Arguments
///
/// * `a`, `b` — limb slices, either order, either length.
///
/// # Returns
///
/// The sum's limbs, one longer than the longest input when the top limb
/// carries. Not trimmed; the caller normalizes.
pub(crate) fn add_limbs(a: &[u64], b: &[u64]) -> Vec<u64> {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    let mut out = Vec::with_capacity(long.len() + 1);
    let mut carry = 0u128;
    for (index, limb) in long.iter().enumerate() {
        let sum = *limb as u128 + short.get(index).copied().unwrap_or(0) as u128 + carry;
        out.push(sum as u64);
        carry = sum >> 64;
    }
    if carry != 0 {
        out.push(carry as u64);
    }
    out
}

/// Subtract `b` from `a` over little-endian limb slices.
///
/// # Arguments
///
/// * `a` — the minuend; the caller must have established `a >= b`.
/// * `b` — the subtrahend, of any length up to `a`'s.
///
/// # Returns
///
/// `a - b`'s limbs, `a.len()` long and not trimmed. When `a < b` the result
/// wraps modulo `2^(64 * a.len())`, which is why the only public entry point,
/// [`BigUint::sub`](crate::bigint::BigUint::sub), compares first and reports
/// [`BigUintError::Underflow`](crate::bigint::BigUintError::Underflow).
pub(crate) fn sub_limbs(a: &[u64], b: &[u64]) -> Vec<u64> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = false;
    for (index, limb) in a.iter().enumerate() {
        let (step, wrapped_operand) = limb.overflowing_sub(b.get(index).copied().unwrap_or(0));
        let (diff, wrapped_borrow) = step.overflowing_sub(u64::from(borrow));
        out.push(diff);
        borrow = wrapped_operand || wrapped_borrow;
    }
    out
}
