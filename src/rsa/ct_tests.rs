//! Unit coverage for the constant-time comparison primitive.
//!
//! [`ct_eq`](super::ct::ct_eq) is the only thing standing between a correct
//! digest check and a leaky one, so its *functional* behaviour is asserted here
//! independently of the PKCS#1 walk that uses it. Timing itself is not asserted
//! — a timing test would be flaky — so the accumulate-then-branch shape in
//! `super::ct` is the load-bearing part and must not be "optimized" into an
//! early return.
//!
//! The integrator wires this with `#[cfg(test)] mod ct_tests;`.

use super::ct::ct_eq;

#[test]
fn equal_slices_compare_equal() {
    assert!(ct_eq(b"", b""));
    assert!(ct_eq(&[0x00], &[0x00]));
    assert!(ct_eq(&[0xff; 64], &[0xff; 64]));
}

#[test]
fn a_difference_at_any_position_is_detected() {
    // Every octet position is checked, so no index can be skipped.
    for position in 0..32usize {
        let left = [0u8; 32];
        let mut right = [0u8; 32];
        right[position] = 0x01;
        assert!(!ct_eq(&left, &right), "difference at {position} missed");
    }
}

#[test]
fn a_single_bit_difference_is_detected() {
    // XOR accumulation must not lose low-weight differences.
    for bit in 0..8u32 {
        assert!(!ct_eq(&[0x00], &[1u8 << bit]));
    }
}

#[test]
fn length_differences_are_rejected_without_prefix_credit() {
    // A shared prefix must not make a shorter slice compare equal, which is what
    // a naive zip-only loop would do.
    assert!(!ct_eq(b"abc", b"abcd"));
    assert!(!ct_eq(b"abcd", b"abc"));
    assert!(!ct_eq(b"", b"a"));
}
