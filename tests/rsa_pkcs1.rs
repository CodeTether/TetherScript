//! Integration coverage for the byte-exact EMSA-PKCS1-v1_5 encoding check.
//!
//! These tests drive `tetherscript::rsa::check_encoding` directly with
//! hand-built encoded messages, so every padding rejection is exercised without
//! needing a private key to produce the corresponding signature. The
//! signature-level and key-level rejections live in `tests/rsa_verify.rs`.
//!
//! Each negative case is a *minimal* mutation of the one known-good block built
//! by [`good`], so a failure points at exactly one rule.
//!
//! # Provenance of the DER prefixes asserted here
//!
//! The `DigestInfo` prefixes are quoted from RFC 8017 section 9.2, note 1. They
//! were independently confirmed against real OpenSSL signatures: signing the
//! same message with `openssl dgst -sha1|-sha256|-sha384|-sha512 -sign` over a
//! 2048-bit key and recovering `s^65537 mod n` yields DigestInfo regions of 35,
//! 51, 67 and 83 octets beginning
//!
//! ```text
//! sha1   3021300906052b0e03021a0500 0414
//! sha256 3031300d060960864801650304020105000420
//! sha384 3041300d060960864801650304020205000430
//! sha512 3051300d060960864801650304020305000440
//! ```

use tetherscript::rsa::{check_encoding, ct_eq, DigestAlgorithm, RsaError};

/// Modulus octet length of a 2048-bit key; the length every block here has.
const K: usize = 256;

/// A digest with no repeated structure, so a misaligned compare cannot pass.
fn digest(alg: DigestAlgorithm) -> Vec<u8> {
    (0..alg.digest_len()).map(|i| (i as u8).wrapping_mul(7) ^ 0x5a).collect()
}

/// Build the one canonical `EM = 0x00 || 0x01 || PS || 0x00 || T` block.
fn good(alg: DigestAlgorithm) -> Vec<u8> {
    let d = digest(alg);
    let mut em = vec![0x00, 0x01];
    em.extend(vec![0xff; K - 3 - alg.encoded_len()]);
    em.push(0x00);
    em.extend_from_slice(alg.der_prefix());
    em.extend_from_slice(&d);
    assert_eq!(em.len(), K, "the canonical block must be exactly k octets");
    em
}

#[test]
fn canonical_encoding_is_accepted_for_every_algorithm() {
    for alg in [
        DigestAlgorithm::Sha1,
        DigestAlgorithm::Sha256,
        DigestAlgorithm::Sha384,
        DigestAlgorithm::Sha512,
    ] {
        assert_eq!(check_encoding(&good(alg), &digest(alg), alg), Ok(()));
    }
}

#[test]
fn digestinfo_prefix_lengths_match_rfc_8017() {
    // 15 octets for SHA-1 (short OID, no NULL-bearing 0x0d header) and 19 for
    // the SHA-2 family. Totals: 35, 51, 67, 83 octets, matching the OpenSSL
    // recovery recorded in the module docs.
    assert_eq!(DigestAlgorithm::Sha1.encoded_len(), 35);
    assert_eq!(DigestAlgorithm::Sha256.encoded_len(), 51);
    assert_eq!(DigestAlgorithm::Sha384.encoded_len(), 67);
    assert_eq!(DigestAlgorithm::Sha512.encoded_len(), 83);
}

#[test]
fn leading_octets_must_be_00_01() {
    let alg = DigestAlgorithm::Sha256;
    // Block type 0x02 is the encryption block type; honouring it here would let
    // an encryption-padded block masquerade as a signature.
    for (first, second) in [(0x00, 0x02), (0x01, 0x01), (0x00, 0x00), (0xff, 0x01)] {
        let mut em = good(alg);
        em[0] = first;
        em[1] = second;
        assert_eq!(
            check_encoding(&em, &digest(alg), alg),
            Err(RsaError::LeadingBytes { first, second }),
            "expected refusal for leading {first:#04x} {second:#04x}"
        );
    }
}

#[test]
fn padding_run_shorter_than_eight_is_refused() {
    // The Bleichenbacher-style lever: truncate PS and hand the freed octets to
    // the attacker. RFC 8017 section 9.2 requires at least 8 octets of 0xff.
    let alg = DigestAlgorithm::Sha256;
    for run in 0..8usize {
        let mut em = vec![0x00, 0x01];
        em.extend(vec![0xff; run]);
        em.push(0x00);
        // Fill the rest with a well-formed DigestInfo so *only* the run is wrong.
        em.extend_from_slice(alg.der_prefix());
        em.extend_from_slice(&digest(alg));
        em.resize(K, 0x00);
        assert_eq!(
            check_encoding(&em, &digest(alg), alg),
            Err(RsaError::PaddingRunTooShort { len: run }),
            "expected refusal for a {run}-octet 0xff run"
        );
    }
}

#[test]
fn a_run_of_exactly_eight_is_the_boundary_and_is_accepted() {
    // Exactly 8 is legal, so the check must be `< 8`, not `<= 8`. This pins the
    // off-by-one in the safe direction: a correct signature over a large digest
    // in a small modulus still verifies.
    let alg = DigestAlgorithm::Sha512;
    let mut em = vec![0x00, 0x01];
    em.extend(vec![0xff; 8]);
    em.push(0x00);
    em.extend_from_slice(alg.der_prefix());
    em.extend_from_slice(&digest(alg));
    assert_eq!(em.len(), 3 + 8 + alg.encoded_len());
    assert_eq!(check_encoding(&em, &digest(alg), alg), Ok(()));
}

#[test]
fn missing_zero_separator_is_refused() {
    let alg = DigestAlgorithm::Sha256;
    let mut em = good(alg);
    // Overwrite the separator with 0x01. Without a mandatory 0x00 the boundary
    // between PS and DigestInfo is attacker-movable.
    let separator = K - alg.encoded_len() - 1;
    assert_eq!(em[separator], 0x00);
    em[separator] = 0x01;
    assert_eq!(check_encoding(&em, &digest(alg), alg), Err(RsaError::MissingSeparator));
}

#[test]
fn an_all_ff_block_with_no_separator_is_refused() {
    // The run walks to the end of the buffer and never finds a terminator.
    let alg = DigestAlgorithm::Sha256;
    let mut em = vec![0x00, 0x01];
    em.resize(K, 0xff);
    assert_eq!(check_encoding(&em, &digest(alg), alg), Err(RsaError::MissingSeparator));
}

#[test]
fn digestinfo_for_the_wrong_hash_is_refused() {
    // Claim SHA-256 but present the SHA-384 DigestInfo, padded to the same
    // overall length so only the algorithm identity differs. Accepting this is
    // algorithm confusion: the attacker, not the caller, picks the hash.
    let claimed = DigestAlgorithm::Sha256;
    let other = DigestAlgorithm::Sha384;
    let mut em = vec![0x00, 0x01];
    em.extend(vec![0xff; K - 3 - claimed.encoded_len()]);
    em.push(0x00);
    em.extend_from_slice(other.der_prefix());
    em.resize(K, 0x00);
    assert_eq!(em.len(), K);
    assert_eq!(
        check_encoding(&em, &digest(claimed), claimed),
        Err(RsaError::DigestInfoMismatch)
    );
}

#[test]
fn sha1_digestinfo_offered_for_a_sha256_claim_is_refused() {
    // The dangerous direction of the same confusion: a cheap-to-collide hash
    // presented where a strong one was requested.
    let claimed = DigestAlgorithm::Sha256;
    let mut em = vec![0x00, 0x01];
    em.extend(vec![0xff; K - 3 - claimed.encoded_len()]);
    em.push(0x00);
    em.extend_from_slice(DigestAlgorithm::Sha1.der_prefix());
    em.resize(K, 0x11);
    assert_eq!(
        check_encoding(&em, &digest(claimed), claimed),
        Err(RsaError::DigestInfoMismatch)
    );
}

#[test]
fn a_one_bit_change_in_the_last_digest_octet_is_refused() {
    // The compare is over the whole digest, not a prefix of it.
    let alg = DigestAlgorithm::Sha256;
    let mut em = good(alg);
    *em.last_mut().unwrap() ^= 0x01;
    assert_eq!(check_encoding(&em, &digest(alg), alg), Err(RsaError::DigestMismatch));
}

#[test]
fn a_one_bit_change_in_the_first_digest_octet_is_refused() {
    let alg = DigestAlgorithm::Sha256;
    let mut em = good(alg);
    em[K - alg.digest_len()] ^= 0x80;
    assert_eq!(check_encoding(&em, &digest(alg), alg), Err(RsaError::DigestMismatch));
}

#[test]
fn a_trailing_octet_after_the_digest_is_refused() {
    // Shorten PS by one and append one attacker-chosen octet, keeping the block
    // at k octets. A verifier that stops reading after the digest accepts this,
    // and that slack is what small-exponent forgeries exploit.
    let alg = DigestAlgorithm::Sha256;
    let mut em = good(alg);
    em.remove(2);
    em.push(0xaa);
    assert_eq!(em.len(), K);
    assert_eq!(
        check_encoding(&em, &digest(alg), alg),
        Err(RsaError::DigestInfoLength { expected: 51, found: 52 })
    );
}

#[test]
fn a_digestinfo_region_one_octet_short_is_refused() {
    // Lengthen PS by one so the region after the separator is 50 octets.
    let alg = DigestAlgorithm::Sha256;
    let mut em = good(alg);
    em.insert(2, 0xff);
    em.truncate(K);
    assert_eq!(
        check_encoding(&em, &digest(alg), alg),
        Err(RsaError::DigestInfoLength { expected: 51, found: 50 })
    );
}

#[test]
fn a_caller_supplied_digest_of_the_wrong_length_is_refused() {
    // Guards the caller's own bug: handing a SHA-1 digest to a SHA-256 verify
    // would otherwise compare 20 octets against a 32-octet field.
    let alg = DigestAlgorithm::Sha256;
    assert_eq!(
        check_encoding(&good(alg), &[0u8; 20], alg),
        Err(RsaError::DigestLength { expected: 32, found: 20 })
    );
}

#[test]
fn a_block_too_small_to_hold_the_structure_is_refused() {
    // A 1024-bit-sized block (128 octets) can hold SHA-512 PKCS#1 v1.5, but a
    // 64-octet one cannot: 3 + 8 + 83 = 94 octets are needed.
    let alg = DigestAlgorithm::Sha512;
    assert_eq!(
        check_encoding(&[0u8; 64], &digest(alg), alg),
        Err(RsaError::EncodingTooShort { modulus_bytes: 64, needed: 94 })
    );
}

#[test]
fn constant_time_compare_agrees_with_equality() {
    // ct_eq is the primitive the digest and DigestInfo comparisons rely on, so
    // its correctness is asserted independently of its timing property.
    assert!(ct_eq(&[], &[]));
    assert!(ct_eq(&[0x00, 0xff], &[0x00, 0xff]));
    assert!(!ct_eq(&[0x00, 0xff], &[0x00, 0xfe]));
    assert!(!ct_eq(&[0x80], &[0x00]));
    assert!(!ct_eq(&[0x01, 0x02], &[0x01]));
    assert!(!ct_eq(&[0x01], &[0x01, 0x02]));
}
