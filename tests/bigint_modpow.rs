//! Integration coverage for `BigUint::modpow`, the RSA primitive.
//!
//! Small cases are checked against textbook values; the 2048-bit case exists to
//! prove the square-and-multiply ladder *terminates* at RSA sizes, which a naive
//! repeated-multiply loop provably would not (65537 multiplications for a public
//! exponent, and on the order of 2^2048 for a private one).

use tetherscript::bigint::{BigUint, BigUintError};

/// A fixed 2048-bit odd modulus, shaped like an RSA `n`: top bit set so the bit
/// length is exactly 2048, low bit set so it is odd. Written as hex because
/// that is how a JWKS modulus arrives.
const MODULUS_2048_HEX: &str = concat!(
    "c1659ecaffdfff059f3602a3448b7f8744fca42bff2a03038eadb735de36453c",
    "6319dd25a55520ca25f107f4dd5a43101b4544f61bfff98f1930f7b8d101e3ca",
    "fdaa81ab52eeec707a949a0874f4f03c036faad08516b56410cf466addec65df",
    "0d75b7cc7ffe352af96c44fde057e3fa32c7148248afed93acfda5ba950d464c",
    "1df73c47e77e72611ff5b18dd11fbc2381350140dfd0bcba63493d0f5de2e22b",
    "6fbe745b9a2e7bf16334e1d044e6a838cd27c544040255367ff9eaa6aee217e0",
    "2cce2d8d3a892672ecc2f7b2e5018c3f8e6dcee05f89ade1ef121e373a35b81a",
    "fc95d30a11a82f13e23b8dafaf2f7591b842da35bf173d20a7c98dfeca9b427b",
);

/// A 256-bit base, the size a SHA-256 digest would occupy before padding.
const BASE_HEX: &str = "a7bbb994edc9ff01b7da78ef3d55a2acef7afde3f5fd631d6fd3843ee4c587c8";

fn modulus_2048() -> BigUint {
    let modulus = BigUint::from_hex(MODULUS_2048_HEX).expect("fixture must be valid hex");
    assert_eq!(modulus.bit_len(), 2048, "the fixture must really be 2048 bits");
    modulus
}

#[test]
fn modpow_matches_the_textbook_case_four_to_the_thirteenth_mod_497() {
    // 4^13 == 67108864, and 67108864 mod 497 == 445.
    assert_eq!(
        BigUint::from_u64(4)
            .modpow(&BigUint::from_u64(13), &BigUint::from_u64(497))
            .unwrap(),
        BigUint::from_u64(445)
    );
}

#[test]
fn modpow_matches_two_to_the_tenth_mod_1000() {
    // 2^10 == 1024, and 1024 mod 1000 == 24.
    assert_eq!(
        BigUint::from_u64(2)
            .modpow(&BigUint::from_u64(10), &BigUint::from_u64(1_000))
            .unwrap(),
        BigUint::from_u64(24)
    );
}

#[test]
fn modpow_with_exponent_zero_is_one() {
    let modulus = BigUint::from_u64(1_000);
    assert!(BigUint::from_u64(2).modpow(&BigUint::zero(), &modulus).unwrap().is_one());
    // Including for a base of zero: this implementation follows the 0^0 == 1
    // convention that RSA's algebra assumes.
    assert!(BigUint::zero().modpow(&BigUint::zero(), &modulus).unwrap().is_one());
    // And for a 2048-bit modulus, where the accumulator is never squared.
    assert!(
        BigUint::from_hex(BASE_HEX)
            .unwrap()
            .modpow(&BigUint::zero(), &modulus_2048())
            .unwrap()
            .is_one()
    );
}

#[test]
fn modpow_with_exponent_one_is_the_base_reduced() {
    let one = BigUint::from_u64(1);
    let modulus = BigUint::from_u64(1_000);
    assert_eq!(BigUint::from_u64(2).modpow(&one, &modulus).unwrap(), BigUint::from_u64(2));
    // 1234 mod 1000 == 234: the base is reduced first, so it may exceed the
    // modulus on the way in.
    assert_eq!(
        BigUint::from_u64(1_234).modpow(&one, &modulus).unwrap(),
        BigUint::from_u64(234)
    );
}

#[test]
fn modpow_with_modulus_one_is_always_zero() {
    // Every residue class collapses, including for a zero exponent, since the
    // initial accumulator is itself reduced.
    let one = BigUint::from_u64(1);
    assert!(BigUint::from_u64(5).modpow(&BigUint::from_u64(3), &one).unwrap().is_zero());
    assert!(BigUint::from_u64(5).modpow(&BigUint::zero(), &one).unwrap().is_zero());
}

#[test]
fn modpow_by_a_zero_modulus_is_a_named_error() {
    assert_eq!(
        BigUint::from_u64(2).modpow(&BigUint::from_u64(3), &BigUint::zero()),
        Err(BigUintError::DivideByZero)
    );
    // The error fires even with a zero exponent, because reducing the initial
    // accumulator already needs the modulus.
    assert_eq!(
        BigUint::from_u64(2).modpow(&BigUint::zero(), &BigUint::zero()),
        Err(BigUintError::DivideByZero)
    );
}

#[test]
fn modpow_matches_repeated_multiplication_for_a_small_exponent() {
    // Cross-check the ladder against the naive definition, which is only
    // tractable because the exponent here is tiny.
    let base = BigUint::from_u64(7);
    let modulus = BigUint::from_u64(13);
    let mut expected = BigUint::from_u64(1);
    for _ in 0..40 {
        expected = expected.mulmod(&base, &modulus).unwrap();
    }
    assert_eq!(base.modpow(&BigUint::from_u64(40), &modulus).unwrap(), expected);
}

#[test]
fn modpow_handles_an_exponent_wider_than_one_limb() {
    // 3^(2^64) mod 5. The order of 3 modulo 5 is 4, and 2^64 is divisible by 4,
    // so the result is 1. An exponent this large is exactly the case a naive
    // loop cannot reach.
    let exponent = BigUint::from_limbs_le(vec![0, 1]);
    assert_eq!(exponent.bit_len(), 65);
    assert!(BigUint::from_u64(3).modpow(&exponent, &BigUint::from_u64(5)).unwrap().is_one());
}

#[test]
fn modpow_reduces_across_limb_boundaries() {
    // (2^63)^5 == 2^315. Modulo 2^64 + 1 we have 2^64 == -1, so
    // 2^315 == 2^(64*4 + 59) == (-1)^4 * 2^59 == 2^59 == 576460752303423488.
    let result = BigUint::from_u64(1 << 63)
        .modpow(&BigUint::from_u64(5), &BigUint::from_limbs_le(vec![1, 1]))
        .unwrap();
    assert_eq!(result, BigUint::from_u64(576_460_752_303_423_488));
}

#[test]
fn modpow_completes_a_2048_bit_exponentiation_with_the_public_exponent() {
    // The load-bearing test: 2048-bit modulus, e = 65537, which is 17 exponent
    // bits and therefore 17 squarings plus 2 multiplies. A naive loop would need
    // 65537 multiplications here and 2^2048 for a private exponent.
    let modulus = modulus_2048();
    let base = BigUint::from_hex(BASE_HEX).unwrap();
    let result = base.modpow(&BigUint::from_u64(65_537), &modulus).unwrap();

    assert!(result < modulus, "the result must be reduced");
    assert_eq!(result.bit_len(), 2048, "this particular result fills the modulus");
    // Fixed expected value, computed independently and pinned here so a
    // regression in carry, division, or reduction cannot pass unnoticed.
    assert_eq!(
        result.to_hex(),
        concat!(
            "84d1f13ecccd6d610720581bcc9c34c4aa9c50f77721d39087e37c8b8b9a975b",
            "a809dbeade89cc1e60cdccb0b7ea74cf6ffbfbd04b16d40df753fbb3ce1ab18f",
            "85f2730f1dd177f98ecadaf6c57dbabf444d2f32141075300ea39474735a2194",
            "17526982b597f95c85a03e6fa0476a6b1c6db30643b100762c163a561c9ef850",
            "60884dae350d742422b0b02e5f232aa98a299928c3fe00d8a52990ea3b82a51c",
            "ba2a97cd485fa17a1f08dc16f73f30b65fe3225c34c5a0638cc9b80fdff0c491",
            "e248447fc8a303f35e149cfef58dba563104ee7448f4405ae54959438ae94620",
            "7bba411a43f4647b96f510ed9b33dfc9788f3723e6d02c5c8af3b308e26225e4",
        )
    );
    // I2OSP at the modulus width, the form an RSA signature takes on the wire.
    assert_eq!(result.to_be_bytes(256).unwrap().len(), 256);
}

#[test]
fn modpow_agrees_with_an_explicit_square_and_multiply_ladder_at_2048_bits() {
    // Independent of the pinned vector: 65537 == 2^16 + 1, so m^65537 is m
    // squared sixteen times and then multiplied by m once.
    let modulus = modulus_2048();
    let base = BigUint::from_hex(BASE_HEX).unwrap();
    let mut expected = base.clone();
    for _ in 0..16 {
        expected = expected.mulmod(&expected, &modulus).unwrap();
    }
    expected = expected.mulmod(&base, &modulus).unwrap();
    assert_eq!(base.modpow(&BigUint::from_u64(65_537), &modulus).unwrap(), expected);
}

#[test]
fn modpow_is_multiplicative_in_the_exponent_at_2048_bits() {
    // m^(a+b) == m^a * m^b mod n, a structural identity that does not depend on
    // any pinned digits.
    let modulus = modulus_2048();
    let base = BigUint::from_hex(BASE_HEX).unwrap();
    let (a, b) = (BigUint::from_u64(9_973), BigUint::from_u64(55_564));
    let combined = base.modpow(&a.add(&b), &modulus).unwrap();
    let split = base
        .modpow(&a, &modulus)
        .unwrap()
        .mulmod(&base.modpow(&b, &modulus).unwrap(), &modulus)
        .unwrap();
    assert_eq!(combined, split);
}

#[test]
fn modpow_squaring_at_2048_bits_matches_direct_multiplication() {
    let modulus = modulus_2048();
    let base = BigUint::from_hex(BASE_HEX).unwrap();
    assert_eq!(
        base.modpow(&BigUint::from_u64(2), &modulus).unwrap(),
        base.mulmod(&base, &modulus).unwrap()
    );
}
