//! Integration tests for the `bignum` arbitrary-precision unsigned integer.
//!
//! Coverage is organized as: hand-verified small arithmetic, limb-boundary
//! crossings, multi-limb long division, an algebraic round-trip
//! (`a * b / b == a`) over many pseudo-random values, `mod_pow` including its
//! zero/one edge cases, a real 2048-bit exponentiation checked by Fermat's
//! little theorem (self-checking — no external oracle needed), string and byte
//! round-trips, and the documented subtraction-underflow behaviour.

use tetherscript::bignum::{ParseUintError, Uint};

/// The 2048-bit MODP group 14 prime from RFC 3526 §3. It is a published,
/// independently verifiable safe prime, which makes `2^(p-1) mod p == 1` a
/// self-checking exercise of the full multi-limb path.
const RFC3526_2048: &str = "\
    FFFFFFFF FFFFFFFF C90FDAA2 2168C234 C4C6628B 80DC1CD1 \
    29024E08 8A67CC74 020BBEA6 3B139B22 514A0879 8E3404DD \
    EF9519B3 CD3A431B 302B0A6D F25F1437 4FE1356D 6D51C245 \
    E485B576 625E7EC6 F44C42E9 A637ED6B 0BFF5CB6 F406B7ED \
    EE386BFB 5A899FA5 AE9F2411 7C4B1FE6 49286651 ECE45B3D \
    C2007CB8 A163BF05 98DA4836 1C55D39A 69163FA8 FD24CF5F \
    83655D23 DCA3AD96 1C62F356 208552BB 9ED52907 7096966D \
    670C354E 4ABC9804 F1746C08 CA18217C 32905E46 2E36CE3B \
    E39E772C 180E8603 9B2783A2 EC07A28F B5C55DF0 6F4C52C9 \
    DE2BCBF6 95581718 3995497C EA956AE5 15D22618 98FA0510 \
    15728E5A 8AACAA68 FFFFFFFF FFFFFFFF";

fn u(value: u64) -> Uint {
    Uint::from_u64(value)
}

fn dec(text: &str) -> Uint {
    Uint::from_dec_str(text).expect("test literal must be valid decimal")
}

fn hex(text: &str) -> Uint {
    Uint::from_hex_str(text).expect("test literal must be valid hex")
}

/// A tiny deterministic PRNG (the PCG/Knuth LCG multiplier) so the randomized
/// tests are reproducible and pull in no dependency.
struct Lcg(u64);

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    /// Builds a value of up to `limbs` limbs (possibly fewer after
    /// normalization).
    fn next_uint(&mut self, limbs: usize) -> Uint {
        Uint::from_limbs((0..limbs).map(|_| self.next_u64()).collect())
    }
}

#[test]
fn zero_and_one_are_normalized() {
    assert!(Uint::zero().is_zero());
    assert_eq!(Uint::zero().limbs(), &[] as &[u64]);
    assert_eq!(Uint::from_u64(0), Uint::zero());
    assert_eq!(Uint::from_limbs(vec![0, 0, 0]), Uint::zero());
    assert_eq!(Uint::from_limbs(vec![7, 0, 0]), u(7));
    assert!(Uint::one().is_one());
    assert!(!Uint::zero().is_one());
    assert_eq!(Uint::default(), Uint::zero());
}

#[test]
fn small_arithmetic_verified_by_hand() {
    assert_eq!(u(2).add(&u(3)), u(5));
    assert_eq!(u(255).add(&u(1)), u(256));
    assert_eq!(u(9).sub(&u(4)), u(5));
    assert_eq!(u(6).mul(&u(7)), u(42));
    assert_eq!(u(17).divmod(&u(5)), (u(3), u(2)));
    assert_eq!(u(100).div(&u(7)), u(14));
    assert_eq!(u(100).rem(&u(7)), u(2));
    // Anything divided by one is itself, with no remainder.
    assert_eq!(u(12345).divmod(&Uint::one()), (u(12345), Uint::zero()));
    // Zero on either side of a product is zero.
    assert_eq!(u(12345).mul(&Uint::zero()), Uint::zero());
    assert_eq!(Uint::zero().mul(&u(12345)), Uint::zero());
    assert_eq!(Uint::zero().add(&u(8)), u(8));
    assert_eq!(u(8).sub(&u(8)), Uint::zero());
}

#[test]
fn comparison_respects_magnitude_not_limb_order() {
    let one_limb_max = u(u64::MAX);
    let two_limbs = Uint::from_limbs(vec![0, 1]);
    assert!(one_limb_max < two_limbs);
    assert!(two_limbs > one_limb_max);
    assert_eq!(two_limbs.cmp_uint(&two_limbs), std::cmp::Ordering::Equal);
    assert!(Uint::zero() < Uint::one());
    // Low limb differs only: still ordered correctly.
    assert!(Uint::from_limbs(vec![1, 5]) < Uint::from_limbs(vec![2, 5]));
    assert!(Uint::from_limbs(vec![u64::MAX, 5]) < Uint::from_limbs(vec![0, 6]));
    let mut sorted = vec![u(9), Uint::zero(), two_limbs.clone(), u(1)];
    sorted.sort();
    assert_eq!(sorted, vec![Uint::zero(), u(1), u(9), two_limbs]);
}

#[test]
fn addition_carries_across_limbs() {
    assert_eq!(u(u64::MAX).add(&Uint::one()).limbs(), &[0, 1]);
    // 2^64 - 1 + 2^64 - 1 == 2^65 - 2
    let doubled = u(u64::MAX).add(&u(u64::MAX));
    assert_eq!(doubled.limbs(), &[u64::MAX - 1, 1]);
    // A carry rippling through a full limb of ones.
    let ripple = Uint::from_limbs(vec![u64::MAX, u64::MAX]).add(&Uint::one());
    assert_eq!(ripple.limbs(), &[0, 0, 1]);
    assert_eq!(ripple.bit_len(), 129);
}

#[test]
fn multiplication_crosses_the_limb_boundary() {
    // 2^32 * 2^32 == 2^64, which must become the second limb, not wrap to zero.
    let root = u(1 << 32);
    assert_eq!(root.mul(&root).limbs(), &[0, 1]);
    // The worst case for the u128 intermediate: (2^64 - 1)^2.
    let max = u(u64::MAX);
    let square = max.mul(&max);
    assert_eq!(square.limbs(), &[1, u64::MAX - 1]);
    assert_eq!(
        square.to_dec_string(),
        "340282366920938463426481119284349108225"
    );
    // 2^64 * 2^64 == 2^128.
    let two64 = Uint::from_limbs(vec![0, 1]);
    assert_eq!(two64.mul(&two64).limbs(), &[0, 0, 1]);
    assert_eq!(two64.mul(&two64).bit_len(), 129);
}

#[test]
fn multiplication_matches_decimal_reference() {
    let a = dec("123456789012345678901234567890");
    let b = dec("987654321098765432109876543210");
    assert_eq!(
        a.mul(&b).to_dec_string(),
        "121932631137021795226185032733622923332237463801111263526900"
    );
    assert_eq!(a.mul(&b), b.mul(&a));
}

#[test]
fn division_with_a_multi_limb_divisor() {
    // Six-limb dividend by a two-limb divisor: the Knuth Algorithm D path.
    let a = hex("123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0\
                 123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0");
    let b = hex("FEDCBA9876543210FFFF");
    let (q, r) = a.divmod(&b);
    assert_eq!(
        q.to_dec_string(),
        "2328041115810841211025066029384464048689512112487485744274755959626133059528599962436789289"
    );
    assert_eq!(r.to_dec_string(), "1154504593564910280262425");
    assert!(r < b);
    assert_eq!(q.mul(&b).add(&r), a);
}

#[test]
fn division_edge_shapes() {
    // Divisor larger than the dividend.
    assert_eq!(u(5).divmod(&u(17)), (Uint::zero(), u(5)));
    // Exactly equal operands.
    let big = dec("340282366920938463463374607431768211455");
    assert_eq!(big.divmod(&big), (Uint::one(), Uint::zero()));
    // Zero dividend.
    assert_eq!(Uint::zero().divmod(&big), (Uint::zero(), Uint::zero()));
    // Divisor whose top limb is already normalized (high bit set), so
    // Algorithm D's shift is zero.
    let d = Uint::from_limbs(vec![1, 1 << 63]);
    let n = Uint::from_limbs(vec![u64::MAX, u64::MAX, u64::MAX]);
    let (q, r) = n.divmod(&d);
    assert!(r < d);
    assert_eq!(q.mul(&d).add(&r), n);
    // Power-of-two divisor: division must agree with a shift.
    let value = Uint::from_limbs(vec![0xDEAD_BEEF, 0xCAFE_BABE, 7]);
    assert_eq!(value.div(&Uint::one().shl(70)), value.shr(70));
}

#[test]
#[should_panic(expected = "Uint division by zero")]
fn division_by_zero_panics() {
    let _ = u(1).divmod(&Uint::zero());
}

#[test]
#[should_panic(expected = "Uint division by zero")]
fn single_limb_division_by_zero_panics() {
    let _ = u(1).divmod_u64(0);
}

#[test]
fn mul_then_div_round_trips_over_many_values() {
    let mut rng = Lcg(0x5EED_1234_ABCD_9876);
    for round in 0..400 {
        let a = rng.next_uint(1 + round % 5);
        let b = rng.next_uint(1 + (round / 3) % 4);
        if b.is_zero() {
            continue;
        }
        // a * b / b == a, exactly, with no remainder.
        let product = a.mul(&b);
        let (q, r) = product.divmod(&b);
        assert_eq!(q, a, "a*b/b != a for a={a} b={b}");
        assert!(r.is_zero(), "a*b % b != 0 for a={a} b={b}");
        // And the general division invariant on an unrelated pair.
        let (q2, r2) = a.divmod(&b);
        assert!(r2 < b, "remainder not reduced for a={a} b={b}");
        assert_eq!(q2.mul(&b).add(&r2), a, "q*b+r != a for a={a} b={b}");
    }
}

#[test]
fn divmod_matches_u64_path_for_single_limb_divisors() {
    let mut rng = Lcg(0x1357_9BDF_2468_ACE0);
    for _ in 0..200 {
        let a = rng.next_uint(4);
        let d = rng.next_u64() | 1;
        let (q, r) = a.divmod(&u(d));
        let (q2, r2) = a.divmod_u64(d);
        assert_eq!(q, q2);
        assert_eq!(r, u(r2));
        assert_eq!(q.mul_u64(d).add(&r), a);
    }
}

#[test]
fn subtraction_underflow_is_reported_not_wrapped() {
    // Documented contract: checked_sub yields None rather than wrapping.
    assert_eq!(u(3).checked_sub(&u(5)), None);
    assert_eq!(Uint::zero().checked_sub(&Uint::one()), None);
    let big = dec("100000000000000000000000000000");
    assert_eq!(u(1).checked_sub(&big), None);
    // Successful cases, including a borrow across the limb boundary.
    assert_eq!(u(5).checked_sub(&u(3)), Some(u(2)));
    assert_eq!(
        Uint::from_limbs(vec![0, 1]).checked_sub(&Uint::one()),
        Some(u(u64::MAX))
    );
    assert_eq!(
        Uint::from_limbs(vec![0, 0, 1]).checked_sub(&Uint::one()),
        Some(Uint::from_limbs(vec![u64::MAX, u64::MAX]))
    );
    // Equal operands normalize back to the canonical zero.
    assert_eq!(big.checked_sub(&big), Some(Uint::zero()));
    assert!(big.sub(&big).limbs().is_empty());
}

#[test]
#[should_panic(expected = "Uint underflow")]
fn sub_panics_on_underflow() {
    let _ = u(3).sub(&u(5));
}

#[test]
fn bit_length_and_bit_tests() {
    assert_eq!(Uint::zero().bit_len(), 0);
    assert_eq!(Uint::one().bit_len(), 1);
    assert_eq!(u(255).bit_len(), 8);
    assert_eq!(u(256).bit_len(), 9);
    assert_eq!(u(u64::MAX).bit_len(), 64);
    assert_eq!(Uint::from_limbs(vec![0, 1]).bit_len(), 65);
    let five = u(0b101);
    assert!(five.bit(0));
    assert!(!five.bit(1));
    assert!(five.bit(2));
    // Out-of-range bits read as zero rather than panicking.
    assert!(!five.bit(63));
    assert!(!five.bit(64));
    assert!(!five.bit(100_000));
    // A bit high in the second limb.
    let high = Uint::one().shl(127);
    assert!(high.bit(127));
    assert!(!high.bit(126));
    assert_eq!(high.bit_len(), 128);
    assert!(u(3).is_odd());
    assert!(!u(4).is_odd());
    assert!(!Uint::zero().is_odd());
}

#[test]
fn shifts_are_exact_powers_of_two() {
    assert_eq!(Uint::one().shl(64).limbs(), &[0, 1]);
    assert_eq!(u(3).shl(2), u(12));
    assert_eq!(Uint::zero().shl(999), Uint::zero());
    assert_eq!(u(12).shr(2), u(3));
    // Right shift floors.
    assert_eq!(u(13).shr(2), u(3));
    assert_eq!(u(5).shr(99), Uint::zero());
    assert_eq!(Uint::from_limbs(vec![0, 1]).shr(64), Uint::one());
    // shl then shr is the identity; the reverse loses the low bits.
    let value = dec("987654321098765432109876543210");
    for bits in [0usize, 1, 7, 63, 64, 65, 130, 191] {
        assert_eq!(value.shl(bits).shr(bits), value, "bits={bits}");
        assert_eq!(value.shl(bits), value.mul(&Uint::one().shl(bits)));
        assert_eq!(value.shr(bits), value.div(&Uint::one().shl(bits)));
    }
    assert_eq!(
        Uint::one().shl(127).to_dec_string(),
        "170141183460469231731687303715884105728"
    );
}

#[test]
fn byte_round_trip_big_endian() {
    assert_eq!(Uint::from_be_bytes(&[]), Uint::zero());
    assert!(Uint::zero().to_be_bytes().is_empty());
    // Leading zeros are accepted on input and dropped on minimal output.
    assert_eq!(Uint::from_be_bytes(&[0, 0, 1]), Uint::one());
    assert_eq!(Uint::from_be_bytes(&[1, 0]), u(256));
    assert_eq!(u(256).to_be_bytes(), vec![1, 0]);
    assert_eq!(Uint::one().to_be_bytes_padded(4), vec![0, 0, 0, 1]);
    assert_eq!(Uint::zero().to_be_bytes_padded(3), vec![0, 0, 0]);
    // Nine bytes: forces the second limb.
    let nine = [0x01u8, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x11];
    let value = Uint::from_be_bytes(&nine);
    assert_eq!(value.to_be_bytes(), nine.to_vec());
    assert_eq!(value.to_be_bytes_padded(12)[3..].to_vec(), nine.to_vec());
    assert_eq!(value.to_be_bytes_padded(12)[..3].to_vec(), vec![0u8, 0, 0]);
    // The 2048-bit prime survives a byte round trip at its exact width.
    let p = hex(RFC3526_2048);
    assert_eq!(p.bit_len(), 2048);
    let bytes = p.to_be_bytes_padded(256);
    assert_eq!(bytes.len(), 256);
    assert_eq!(Uint::from_be_bytes(&bytes), p);
    // Randomized round trips.
    let mut rng = Lcg(0xFEED_FACE_DEAD_BEEF);
    for _ in 0..200 {
        let v = rng.next_uint(5);
        assert_eq!(Uint::from_be_bytes(&v.to_be_bytes()), v);
        assert_eq!(Uint::from_be_bytes(&v.to_be_bytes_padded(64)), v);
    }
}

#[test]
#[should_panic(expected = "bytes but only")]
fn padded_bytes_reject_truncation() {
    let _ = u(256).to_be_bytes_padded(1);
}

#[test]
fn decimal_round_trip_and_errors() {
    assert_eq!(Uint::zero().to_dec_string(), "0");
    assert_eq!(dec("007"), u(7));
    assert_eq!(dec("18446744073709551616").limbs(), &[0, 1]);
    assert_eq!(
        Uint::from_limbs(vec![0, 1]).to_dec_string(),
        "18446744073709551616"
    );
    // A value whose second nineteen-digit chunk is all zeros: catches missing
    // zero-padding in the chunked formatter.
    let padded = dec("10000000000000000000");
    assert_eq!(padded.to_dec_string(), "10000000000000000000");
    let long = "123456789012345678901234567890123456789012345678901234567890";
    assert_eq!(dec(long).to_dec_string(), long);
    // Display and FromStr agree with the explicit helpers.
    assert_eq!(dec(long).to_string(), long);
    assert_eq!(long.parse::<Uint>().unwrap(), dec(long));
    // Whitespace and underscores are separators, not digits.
    assert_eq!(dec("1_000 000"), u(1_000_000));
    // Errors name the offending character and position.
    assert_eq!(Uint::from_dec_str(""), Err(ParseUintError::Empty));
    assert_eq!(
        Uint::from_dec_str("12x4"),
        Err(ParseUintError::InvalidDigit { ch: 'x', index: 2 })
    );
    assert!(Uint::from_dec_str("-1").is_err());
    assert!(Uint::from_dec_str("1.5").is_err());
    let mut rng = Lcg(0x0BAD_C0DE_0BAD_C0DE);
    for _ in 0..200 {
        let v = rng.next_uint(6);
        assert_eq!(dec(&v.to_dec_string()), v);
    }
}

#[test]
fn hex_round_trip_and_errors() {
    assert_eq!(Uint::zero().to_hex_string(), "0");
    assert_eq!(hex("ff"), u(255));
    assert_eq!(hex("0xFF"), u(255));
    assert_eq!(hex("0XFF"), u(255));
    assert_eq!(u(255).to_hex_string(), "ff");
    assert_eq!(format!("{:x}", u(255)), "ff");
    // Interior limbs must be zero-padded to sixteen digits.
    assert_eq!(
        Uint::from_limbs(vec![0, 1]).to_hex_string(),
        "10000000000000000"
    );
    assert_eq!(
        Uint::from_limbs(vec![1, 1]).to_hex_string(),
        "10000000000000001"
    );
    // Spaced hex, as printed in RFCs, parses verbatim.
    assert_eq!(hex("FFFF FFFF"), u(0xFFFF_FFFF));
    let p = hex(RFC3526_2048);
    assert_eq!(hex(&p.to_hex_string()), p);
    assert_eq!(p.to_hex_string().len(), 512);
    assert!(p.to_hex_string().starts_with("ffffffffffffffffc90fdaa2"));
    assert_eq!(Uint::from_hex_str(""), Err(ParseUintError::Empty));
    assert_eq!(Uint::from_hex_str("0x"), Err(ParseUintError::Empty));
    assert_eq!(
        Uint::from_hex_str("abcg"),
        Err(ParseUintError::InvalidDigit { ch: 'g', index: 3 })
    );
    // Hex and decimal agree on the same value.
    assert_eq!(hex("DE0B6B3A7640000"), dec("1000000000000000000"));
    let mut rng = Lcg(0x1234_5678_9ABC_DEF0);
    for _ in 0..200 {
        let v = rng.next_uint(6);
        assert_eq!(hex(&v.to_hex_string()), v);
    }
}

#[test]
fn mod_pow_small_cases_computable_by_hand() {
    // 3^4 = 81, 81 mod 5 = 1.
    assert_eq!(Uint::mod_pow(&u(3), &u(4), &u(5)), Uint::one());
    // 7^2 = 49, 49 mod 11 = 5.
    assert_eq!(u(7).pow_mod(&u(2), &u(11)), u(5));
    // 2^10 = 1024, 1024 mod 1000 = 24.
    assert_eq!(Uint::mod_pow(&u(2), &u(10), &u(1000)), u(24));
    // 123^11 mod 1000000007, cross-checked against modular reduction below.
    let m = u(1_000_000_007);
    let mut expected = Uint::one();
    for _ in 0..11 {
        expected = expected.mul_mod(&u(123), &m);
    }
    assert_eq!(Uint::mod_pow(&u(123), &u(11), &m), expected);
    assert_eq!(expected, u(308_484_729));
    // Base larger than the modulus is reduced first.
    assert_eq!(Uint::mod_pow(&u(100), &u(1), &u(7)), u(2));
    // Zero base with a positive exponent.
    assert_eq!(Uint::mod_pow(&Uint::zero(), &u(5), &u(7)), Uint::zero());
    // Repeated-multiplication reference for a spread of small triples.
    for base in 0u64..12 {
        for exp in 0u64..12 {
            for modulus in 1u64..12 {
                let mut want = Uint::one().rem(&u(modulus));
                for _ in 0..exp {
                    want = want.mul_mod(&u(base), &u(modulus));
                }
                assert_eq!(
                    Uint::mod_pow(&u(base), &u(exp), &u(modulus)),
                    want,
                    "{base}^{exp} mod {modulus}"
                );
            }
        }
    }
}

#[test]
fn mod_pow_edge_cases() {
    let m = u(7);
    // Exponent zero is one, reduced by the modulus.
    assert_eq!(Uint::mod_pow(&u(5), &Uint::zero(), &m), Uint::one());
    assert_eq!(Uint::mod_pow(&Uint::zero(), &Uint::zero(), &m), Uint::one());
    // Exponent one is the reduced base.
    assert_eq!(Uint::mod_pow(&u(5), &Uint::one(), &m), u(5));
    assert_eq!(Uint::mod_pow(&u(12), &Uint::one(), &m), u(5));
    // Modulus one collapses everything to zero, including a zero exponent.
    let one = Uint::one();
    assert!(Uint::mod_pow(&u(5), &u(3), &one).is_zero());
    assert!(Uint::mod_pow(&u(5), &Uint::zero(), &one).is_zero());
    assert!(Uint::mod_pow(&Uint::zero(), &Uint::zero(), &one).is_zero());
    // Modulus two: parity of the base.
    assert!(Uint::mod_pow(&u(4), &u(9), &u(2)).is_zero());
    assert_eq!(Uint::mod_pow(&u(5), &u(9), &u(2)), Uint::one());
}

#[test]
#[should_panic(expected = "zero modulus")]
fn mod_pow_with_zero_modulus_panics() {
    let _ = Uint::mod_pow(&u(2), &u(3), &Uint::zero());
}

#[test]
fn mod_pow_satisfies_fermat_for_a_small_prime() {
    // 1_000_003 is prime, so a^(p-1) == 1 (mod p) for every a not divisible by p.
    let p = u(1_000_003);
    let e = p.sub(&Uint::one());
    for base in [2u64, 3, 5, 7, 999_983, 1_000_002] {
        assert_eq!(
            Uint::mod_pow(&u(base), &e, &p),
            Uint::one(),
            "Fermat failed for base {base}"
        );
    }
    // 5^117 mod 19 == 1 because 117 is a multiple of ord(5) = 9.
    assert_eq!(Uint::mod_pow(&u(5), &u(117), &u(19)), Uint::one());
}

#[test]
fn mod_pow_is_multiplicative_over_a_large_modulus() {
    // (a*b)^e == a^e * b^e (mod m) — an internally consistent check that the
    // multi-limb multiply and reduce agree with each other.
    let m = hex(RFC3526_2048);
    let e = u(65_537);
    let a = hex("DEADBEEFCAFEBABE0123456789ABCDEF");
    let b = dec("12345678901234567890");
    let left = Uint::mod_pow(&a.mul(&b), &e, &m);
    let right = Uint::mod_pow(&a, &e, &m).mul_mod(&Uint::mod_pow(&b, &e, &m), &m);
    assert_eq!(left, right);
    assert!(left < m);
}

#[test]
fn mod_pow_2048_bit_fermat() {
    // The real path: a 2048-bit modulus and a 2048-bit exponent. RFC 3526's
    // group 14 modulus is prime, so Fermat's little theorem makes this
    // self-checking without an external oracle.
    let p = hex(RFC3526_2048);
    assert_eq!(p.bit_len(), 2048);
    assert_eq!(p.limbs().len(), 32);
    let e = p.sub(&Uint::one());
    assert_eq!(e.bit_len(), 2048);
    assert_eq!(Uint::mod_pow(&u(2), &e, &p), Uint::one());
    // a^p == a (mod p) is the other form of the same theorem.
    let a = hex("DEADBEEFCAFEBABE".repeat(4).as_str());
    assert_eq!(a.bit_len(), 256);
    assert_eq!(Uint::mod_pow(&a, &p, &p), a.rem(&p));
}

#[test]
fn to_u64_and_limb_accessors() {
    assert_eq!(Uint::zero().to_u64(), Some(0));
    assert_eq!(u(9).to_u64(), Some(9));
    assert_eq!(u(u64::MAX).to_u64(), Some(u64::MAX));
    assert_eq!(Uint::from_limbs(vec![0, 1]).to_u64(), None);
    assert_eq!(u(5).limbs(), &[5]);
    assert_eq!(u(21).mul_u64(2), u(42));
    assert_eq!(u(10).add_u64(5), u(15));
}

#[test]
fn modular_helpers_reduce_their_results() {
    let m = u(97);
    assert_eq!(u(10).mod_reduce(&u(7)), u(3));
    assert!(u(10).mod_reduce(&Uint::one()).is_zero());
    assert_eq!(u(50).mul_mod(&u(50), &m), u(2500 % 97));
    assert_eq!(u(5).add_mod(&u(4), &u(7)), u(2));
    // Products far larger than the modulus still reduce correctly.
    let big = dec("340282366920938463463374607431768211455");
    let reduced = big.mul_mod(&big, &m);
    assert!(reduced < m);
    assert_eq!(reduced, big.mul(&big).rem(&m));
}
