//! Integration coverage for `tetherscript::bigint::BigUint` arithmetic.
//!
//! Every expected value here is hand-computed and written out in the comment
//! above the assertion, so the test suite does not depend on the implementation
//! it is checking. Modular exponentiation lives in `tests/bigint_modpow.rs`.

use tetherscript::bigint::{BigUint, BigUintError};

/// 2^64, the smallest two-limb value; the boundary every carry test hinges on.
fn two_pow_64() -> BigUint {
    BigUint::from_limbs_le(vec![0, 1])
}

#[test]
fn addition_carries_through_several_limbs() {
    // (2^192 - 1) + 1 == 2^192. The carry crosses three limb boundaries and
    // grows the vector, which is the case a per-limb loop without propagation
    // gets wrong.
    let ones = BigUint::from_limbs_le(vec![u64::MAX, u64::MAX, u64::MAX]);
    let sum = ones.add(&BigUint::from_u64(1));
    assert_eq!(sum.limbs(), &[0, 0, 0, 1]);
    assert_eq!(sum.bit_len(), 193);
}

#[test]
fn addition_carry_is_absorbed_without_growing() {
    // (2^64 - 1) + 1 == 2^64: limb 0 wraps to 0 and limb 1 becomes 1.
    assert_eq!(BigUint::from_u64(u64::MAX).add(&BigUint::from_u64(1)), two_pow_64());
}

#[test]
fn addition_is_commutative_and_zero_is_the_identity() {
    let a = BigUint::from_limbs_le(vec![u64::MAX, 3]);
    let b = BigUint::from_u64(9);
    assert_eq!(a.add(&b), b.add(&a));
    assert_eq!(a.add(&BigUint::zero()), a);
}

#[test]
fn subtraction_borrows_across_a_zero_limb() {
    // 2^128 - 1 == [MAX, MAX]. Limb 0 wraps, then the *zero* middle limb wraps
    // too, and only limb 2 absorbs the borrow.
    let two_pow_128 = BigUint::from_limbs_le(vec![0, 0, 1]);
    let result = two_pow_128.sub(&BigUint::from_u64(1)).unwrap();
    assert_eq!(result.limbs(), &[u64::MAX, u64::MAX]);
    assert_eq!(result.bit_len(), 128);
}

#[test]
fn subtraction_borrows_across_two_zero_limbs() {
    // 2^192 - 1 == [MAX, MAX, MAX].
    let two_pow_192 = BigUint::from_limbs_le(vec![0, 0, 0, 1]);
    let result = two_pow_192.sub(&BigUint::from_u64(1)).unwrap();
    assert_eq!(result.limbs(), &[u64::MAX, u64::MAX, u64::MAX]);
}

#[test]
fn subtraction_borrows_a_full_limb_across_a_zero_limb() {
    // 2^128 - (2^64 - 1) == 2^128 - 2^64 + 1 == [1, MAX].
    let two_pow_128 = BigUint::from_limbs_le(vec![0, 0, 1]);
    let result = two_pow_128.sub(&BigUint::from_u64(u64::MAX)).unwrap();
    assert_eq!(result.limbs(), &[1, u64::MAX]);
}

#[test]
fn subtraction_round_trips_with_addition() {
    let a = BigUint::from_limbs_le(vec![u64::MAX, 0, 7]);
    let b = BigUint::from_limbs_le(vec![9, 9]);
    assert_eq!(a.add(&b).sub(&b).unwrap(), a);
}

#[test]
fn subtraction_is_checked_not_saturating() {
    // An unsigned type cannot hold 2 - 5, so it is a named error and emphatically
    // not zero.
    assert_eq!(
        BigUint::from_u64(2).sub(&BigUint::from_u64(5)),
        Err(BigUintError::Underflow)
    );
    assert_eq!(BigUint::zero().sub(&BigUint::from_u64(1)), Err(BigUintError::Underflow));
    // One below the boundary still succeeds.
    assert!(two_pow_64().sub(&BigUint::from_u64(1)).is_ok());
}

#[test]
fn subtracting_equal_values_yields_canonical_zero() {
    let a = BigUint::from_limbs_le(vec![7, 7, 7]);
    let zero = a.sub(&a).unwrap();
    assert!(zero.is_zero());
    assert_eq!(zero.limbs().len(), 0, "no trailing zero limbs may survive");
}

#[test]
fn multiplication_matches_a_hand_computed_two_limb_product() {
    // 12345678901234567890 * 98765432109876543210
    //   == 1219326311370217952237463801111263526900
    // Little-endian limbs of each side and of the product were computed
    // independently.
    let a = BigUint::from_u64(12_345_678_901_234_567_890);
    let b = BigUint::from_limbs_le(vec![6_531_711_741_328_785_130, 5]);
    assert_eq!(
        a.mul(&b).limbs(),
        &[1_331_246_629_686_034_420, 10_759_579_566_687_691_682, 3]
    );
}

#[test]
fn multiplication_saturates_the_widest_single_limb_product() {
    // (2^64 - 1)^2 == 2^128 - 2^65 + 1 == [1, 2^64 - 2], the maximum a u128
    // intermediate must hold. A u64 intermediate loses the whole high limb.
    let max = BigUint::from_u64(u64::MAX);
    assert_eq!(max.mul(&max).limbs(), &[1, u64::MAX - 1]);
}

#[test]
fn multiplication_shifts_limbs_when_multiplying_by_a_power_of_two_limb() {
    // 2^64 * 2^64 == 2^128.
    assert_eq!(two_pow_64().mul(&two_pow_64()).limbs(), &[0, 0, 1]);
}

#[test]
fn multiplication_by_zero_and_one_stay_canonical() {
    let a = BigUint::from_limbs_le(vec![3, 0, 5]);
    assert!(a.mul(&BigUint::zero()).is_zero());
    assert_eq!(a.mul(&BigUint::zero()).limbs().len(), 0);
    assert_eq!(a.mul(&BigUint::from_u64(1)), a);
}

#[test]
fn multiplication_is_commutative_and_distributes_over_addition() {
    let a = BigUint::from_limbs_le(vec![u64::MAX, 2]);
    let b = BigUint::from_limbs_le(vec![7, u64::MAX]);
    let c = BigUint::from_u64(65_537);
    assert_eq!(a.mul(&b), b.mul(&a));
    assert_eq!(a.mul(&b.add(&c)), a.mul(&b).add(&a.mul(&c)));
}

#[test]
fn comparison_orders_by_length_then_by_top_limb() {
    assert!(BigUint::from_u64(u64::MAX) < two_pow_64());
    assert!(BigUint::from_limbs_le(vec![1, 2]) > BigUint::from_limbs_le(vec![u64::MAX, 1]));
    assert_eq!(BigUint::from_limbs_le(vec![5, 0]), BigUint::from_u64(5));
}

#[test]
fn bit_len_and_bit_agree_on_limb_boundaries() {
    assert_eq!(BigUint::zero().bit_len(), 0);
    assert_eq!(BigUint::from_u64(1).bit_len(), 1);
    assert_eq!(BigUint::from_u64(u64::MAX).bit_len(), 64);
    assert_eq!(two_pow_64().bit_len(), 65);
    assert!(two_pow_64().bit(64));
    assert!(!two_pow_64().bit(63));
    assert!(!two_pow_64().bit(1_000), "reads past the top are zero, not a panic");
}

#[test]
fn byte_round_trip_preserves_leading_zeros_at_a_fixed_width() {
    let bytes = [0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];
    let value = BigUint::from_be_bytes(&bytes);
    assert_eq!(value.to_be_bytes(bytes.len()).unwrap(), bytes);
    assert_eq!(value.byte_len(), 9, "leading zeros are not part of the magnitude");
}

#[test]
fn byte_round_trip_crosses_a_limb_boundary() {
    // Nine bytes is two limbs, so the encoder must read limb 1 for byte 8.
    let bytes: Vec<u8> = (1u8..=9).collect();
    let value = BigUint::from_be_bytes(&bytes);
    assert_eq!(value.limbs().len(), 2);
    assert_eq!(value.to_be_bytes(9).unwrap(), bytes);
    assert_eq!(value.to_be_bytes(16).unwrap()[7..], bytes[..]);
}

#[test]
fn all_zero_bytes_decode_to_canonical_zero_and_re_encode() {
    let value = BigUint::from_be_bytes(&[0, 0, 0, 0]);
    assert!(value.is_zero());
    assert_eq!(value.byte_len(), 0);
    assert_eq!(value.to_be_bytes(4).unwrap(), vec![0, 0, 0, 0]);
    assert_eq!(value.to_be_bytes(0).unwrap(), Vec::<u8>::new());
    assert!(BigUint::from_be_bytes(&[]).is_zero());
}

#[test]
fn to_be_bytes_refuses_a_width_narrower_than_the_value() {
    // 0x010203 needs three bytes; asking for two must error rather than hand
    // back 0x0203, which would be a different number.
    let value = BigUint::from_be_bytes(&[0x01, 0x02, 0x03]);
    assert_eq!(
        value.to_be_bytes(2),
        Err(BigUintError::WidthTooSmall { needed: 3, width: 2 })
    );
    assert_eq!(value.to_be_bytes(3).unwrap(), vec![0x01, 0x02, 0x03]);
}

#[test]
fn divmod_with_divisor_greater_than_dividend_yields_zero_and_the_dividend() {
    let (quotient, remainder) = BigUint::from_u64(7).divmod(&BigUint::from_u64(1_000)).unwrap();
    assert!(quotient.is_zero());
    assert_eq!(remainder, BigUint::from_u64(7));
}

#[test]
fn divmod_with_divisor_equal_to_dividend_yields_one_and_zero() {
    let a = BigUint::from_limbs_le(vec![u64::MAX, 0, 12]);
    let (quotient, remainder) = a.divmod(&a).unwrap();
    assert!(quotient.is_one());
    assert!(remainder.is_zero());
}

#[test]
fn divmod_matches_a_hand_computed_multi_limb_division() {
    // (2^192 + 5) / (2^64 + 1) == 2^128 - 2^64 + 1 remainder 4, and
    // 2^128 - 2^64 + 1 is [1, 2^64 - 1] in little-endian limbs.
    let dividend = BigUint::from_limbs_le(vec![5, 0, 0, 1]);
    let divisor = BigUint::from_limbs_le(vec![1, 1]);
    let (quotient, remainder) = dividend.divmod(&divisor).unwrap();
    assert_eq!(quotient.limbs(), &[1, u64::MAX]);
    assert_eq!(remainder, BigUint::from_u64(4));
    // The defining identity: dividend == quotient * divisor + remainder.
    assert_eq!(quotient.mul(&divisor).add(&remainder), dividend);
}

#[test]
fn divmod_by_a_single_limb_matches_native_arithmetic() {
    let (quotient, remainder) = BigUint::from_u64(1_000).divmod(&BigUint::from_u64(7)).unwrap();
    assert_eq!(quotient, BigUint::from_u64(1_000 / 7));
    assert_eq!(remainder, BigUint::from_u64(1_000 % 7));
}

#[test]
fn divmod_reconstructs_the_dividend_for_a_wide_multi_limb_divisor() {
    let dividend = BigUint::from_limbs_le(vec![u64::MAX, 12_345, 0, 9_876_543_210]);
    let divisor = BigUint::from_limbs_le(vec![7, u64::MAX, 3]);
    let (quotient, remainder) = dividend.divmod(&divisor).unwrap();
    assert_eq!(quotient.mul(&divisor).add(&remainder), dividend);
    assert!(remainder < divisor, "the remainder must be reduced");
}

#[test]
fn divmod_of_zero_is_zero_for_any_nonzero_divisor() {
    let (quotient, remainder) = BigUint::zero().divmod(&BigUint::from_u64(5)).unwrap();
    assert!(quotient.is_zero() && remainder.is_zero());
}

#[test]
fn divmod_by_one_is_the_identity() {
    let a = BigUint::from_limbs_le(vec![9, 0, 4]);
    let (quotient, remainder) = a.divmod(&BigUint::from_u64(1)).unwrap();
    assert_eq!(quotient, a);
    assert!(remainder.is_zero());
}

#[test]
fn divmod_by_zero_is_a_named_error_and_not_a_panic() {
    assert_eq!(
        BigUint::from_u64(1).divmod(&BigUint::zero()),
        Err(BigUintError::DivideByZero)
    );
    assert_eq!(BigUint::zero().divmod(&BigUint::zero()), Err(BigUintError::DivideByZero));
    assert_eq!(BigUint::from_u64(1).rem(&BigUint::zero()), Err(BigUintError::DivideByZero));
    assert!(BigUintError::DivideByZero.to_string().contains("divide by zero"));
}

#[test]
fn hex_round_trips_across_limb_boundaries() {
    assert_eq!(BigUint::from_hex("10001").unwrap(), BigUint::from_u64(65_537));
    assert_eq!(two_pow_64().to_hex(), "10000000000000000");
    let wide = "ffffffffffffffff0000000000000001";
    assert_eq!(BigUint::from_hex(wide).unwrap().to_hex(), wide);
    assert_eq!(BigUint::zero().to_hex(), "0");
    assert!(BigUint::from_hex("0xnope").is_none());
}

#[test]
fn hex_and_bytes_describe_the_same_value() {
    let bytes = [0xde, 0xad, 0xbe, 0xef, 0x00, 0x11];
    assert_eq!(BigUint::from_be_bytes(&bytes).to_hex(), "deadbeef0011");
}
