//! Unit coverage for the raw carry and borrow loops in the `carry` module.
//!
//! These assert the two failure modes that a hand-written limb loop gets wrong:
//! a carry that must travel through several limbs, and a borrow that must travel
//! *through a zero limb*. Both are invisible in single-limb tests.
//!
//! The integrator wires this with `#[cfg(test)] mod carry_tests;`.

use super::carry::{add_limbs, sub_limbs};

#[test]
fn carry_propagates_through_every_limb_and_grows() {
    // (2^192 - 1) + 1 == 2^192: the carry crosses three limb boundaries.
    let ones = [u64::MAX, u64::MAX, u64::MAX];
    assert_eq!(add_limbs(&ones, &[1]), vec![0, 0, 0, 1]);
}

#[test]
fn carry_stops_where_it_is_absorbed() {
    // Only limb 0 overflows; limb 1 absorbs the carry and no limb is added.
    assert_eq!(add_limbs(&[u64::MAX, 7], &[1]), vec![0, 8]);
}

#[test]
fn add_is_commutative_across_unequal_lengths() {
    let long = [1u64, 2, 3];
    let short = [u64::MAX];
    assert_eq!(add_limbs(&long, &short), add_limbs(&short, &long));
}

#[test]
fn borrow_propagates_across_a_zero_limb() {
    // 2^128 - 1: limb 0 wraps, then the zero limb 1 wraps too, and limb 2 pays.
    assert_eq!(sub_limbs(&[0, 0, 1], &[1]), vec![u64::MAX, u64::MAX, 0]);
}

#[test]
fn borrow_propagates_across_several_zero_limbs() {
    assert_eq!(sub_limbs(&[0, 0, 0, 1], &[1]), vec![u64::MAX, u64::MAX, u64::MAX, 0]);
}

#[test]
fn borrow_from_the_operand_and_from_the_incoming_borrow_both_count() {
    // limb 0: 0 - MAX wraps; limb 1: 0 - 0 - 1 wraps again.
    assert_eq!(sub_limbs(&[0, 0, 1], &[u64::MAX]), vec![1, u64::MAX, 0]);
}

#[test]
fn subtracting_zero_is_the_identity() {
    assert_eq!(sub_limbs(&[5, 9], &[]), vec![5, 9]);
}
