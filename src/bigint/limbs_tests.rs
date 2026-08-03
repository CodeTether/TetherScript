//! Unit coverage for the normalization invariant in the `limbs` module.
//!
//! Normalization is what makes derived equality and [`BigUint::bit_len`]
//! correct, so it is asserted directly rather than only through arithmetic.
//!
//! The integrator wires this with `#[cfg(test)] mod limbs_tests;`.

use super::limbs::BigUint;

#[test]
fn trailing_zero_limbs_are_trimmed_on_construction() {
    assert_eq!(BigUint::from_limbs_le(vec![7, 0, 0]).limbs(), &[7]);
    assert_eq!(BigUint::from_limbs_le(vec![0, 1, 0]).limbs(), &[0, 1]);
}

#[test]
fn interior_zero_limbs_are_preserved() {
    // 2^128 + 1 genuinely has a zero middle limb; trimming is only from the top.
    assert_eq!(BigUint::from_limbs_le(vec![1, 0, 1]).limbs(), &[1, 0, 1]);
}

#[test]
fn zero_has_exactly_one_representation() {
    assert_eq!(BigUint::from_limbs_le(vec![0, 0, 0]), BigUint::zero());
    assert_eq!(BigUint::from_u64(0), BigUint::zero());
    assert_eq!(BigUint::zero().limbs().len(), 0);
}

#[test]
fn normalization_is_what_makes_equality_meaningful() {
    // Were the trailing zero kept, the derived PartialEq would call these
    // different numbers, which is the failure the invariant exists to prevent.
    assert_eq!(BigUint::from_limbs_le(vec![5, 0]), BigUint::from_u64(5));
}

#[test]
fn bit_len_would_be_inflated_by_an_untrimmed_limb() {
    assert_eq!(BigUint::from_limbs_le(vec![1, 0]).bit_len(), 1);
    assert_eq!(BigUint::from_limbs_le(vec![u64::MAX, 0]).bit_len(), 64);
}

#[test]
fn is_zero_and_is_one_read_the_canonical_forms() {
    assert!(BigUint::from_limbs_le(vec![0]).is_zero());
    assert!(BigUint::from_limbs_le(vec![1, 0]).is_one());
    assert!(!BigUint::from_limbs_le(vec![1, 1]).is_one());
}
