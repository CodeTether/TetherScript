//! Behaviour tests for JWKS parsing and key selection.
//!
//! # Where the fixtures come from
//!
//! The 2048-bit moduli are the real moduli of RSA keys generated with
//! `openssl genrsa 2048`, base64url-encoded and unpadded, so the size and
//! bit-length assertions are about genuine key material rather than a hand-made
//! byte pattern. The 1024-bit one is likewise a real key. `AQAB` is the standard
//! public exponent 65537. The leading-zero and even-modulus fixtures are the
//! `key-a` modulus with a `00` byte prepended and with its low bit cleared
//! respectively, so each differs from an accepted key in exactly the one way the
//! test is about.
//!
//! # What is deliberately not tested here
//!
//! Signature verification. The module under test produces validated key material
//! and performs no RSA arithmetic, so there is no "valid signature" case to
//! assert — only the refusals that must happen before any verifier runs.

use tetherscript::jwks::alg::SigAlg;
use tetherscript::jwks::error::JwksError;
use tetherscript::jwks::keyset::JwkSet;

/// Real 2048-bit modulus of an `openssl genrsa 2048` key, base64url, unpadded.
const N_A: &str = concat!(
    "q9HPapksy45yL75Z_RcMu_f8pPnreongJsZAlGrkeH8nETCOGdJ8wTy08MFsdVsvRjRFfUbpPGVX",
    "CtTS5xSfBh_k5vc5mQPmBLJQOZ21w-T3TxCc4t8CO-MwGF8oe-VIDrnNSfimFu-T50d1Coelbn6_",
    "yq-eJuQQBYqPHW9du9h336oPFaEmLvcfRpmkxuIkiwjX0HzEvxf_eqf3kp76RmIR09NS1tNJU0ZN",
    "yz8VmU_cZHlb3fYAusPNG5V2CaLfHtXgo7DaMK_aAJwnN5hc2ri1d66hkpm8qKPpkbIYSm0O3_lo",
    "1bwMyL3zUXjbfTQSulr43fgXdCbJ7CwINO7dTQ",
);

/// A second, unrelated real 2048-bit modulus, so tie cases have two distinct keys.
const N_B: &str = concat!(
    "nweI2q5sFK2_Cg71kNSHBARlDq720TxRSo9dZF9N5okxWt4_HWUwyTcBr-CEFgxaLkj_eFqWyHb0",
    "y6EDEpmrCidhlSEqRjQ0LEs3-pjp-GlPaNwFfv501gWmd9mNtp2litE1kY_xvVd8u9XHvf_Xf134",
    "-UKL5sTAe2zcNLcmghXEfsWYQLH6-33S2MR5C5vtlRmni4_bpXW--wlF8gh5sp4Fp4d9PMNKgMDC",
    "YGEEIXyJfogQ1KklPNVK9IVgSZBpsPvN80eh-ZuCvDL0z5qZCIMZ69AQYIw6bnfXVgsLyi5UE53k",
    "7zgN0Fo8RGNOtc_Hh0BLz-LOmAsAXTALNo9J0w",
);

/// A real 1024-bit modulus: well-formed, but under the 2048-bit floor.
const N_1024: &str = concat!(
    "nX09gegYfRNa29LYBKvywRlW5DXZpEgkJDHghT1vnJZAh-8Np-R-k4srntFL2Rnkv0fCAAZAJ5Er",
    "eQfRKJPb3CytnF8Ddv14nAfuKBRKbNRRFPZ0fvV4oGAwpuuOCIVFMNimiLx7Q3OAzPPkJaG3DfjR",
    "cKPKqKeOcYXLNVwwVis",
);

/// [`N_A`] with a `00` byte prepended: the same integer, a non-minimal encoding.
const LEADING_ZERO_N: &str = concat!(
    "AKvRz2qZLMuOci--Wf0XDLv3_KT563qJ4CbGQJRq5Hh_JxEwjhnSfME8tPDBbHVbL0Y0RX1G6T",
    "xlVwrU0ucUnwYf5Ob3OZkD5gSyUDmdtcPk908QnOLfAjvjMBhfKHvlSA65zUn4phbvk-dHdQqH",
    "pW5-v8qvnibkEAWKjx1vXbvYd9-qDxWhJi73H0aZpMbiJIsI19B8xL8X_3qn95Ke-kZiEdPTUt",
    "bTSVNGTcs_FZlP3GR5W932ALrDzRuVdgmi3x7V4KOw2jCv2gCcJzeYXNq4tXeuoZKZvKij6ZGy",
    "GEptDt_5aNW8DMi981F42300Erpa-N34F3QmyewsCDTu3U0",
);

/// [`N_A`] with its low bit cleared: 2048 bits, minimally encoded, and even.
const EVEN_N: &str = concat!(
    "q9HPapksy45yL75Z_RcMu_f8pPnreongJsZAlGrkeH8nETCOGdJ8wTy08MFsdVsvRjRFfUbpPG",
    "VXCtTS5xSfBh_k5vc5mQPmBLJQOZ21w-T3TxCc4t8CO-MwGF8oe-VIDrnNSfimFu-T50d1Coel",
    "bn6_yq-eJuQQBYqPHW9du9h336oPFaEmLvcfRpmkxuIkiwjX0HzEvxf_eqf3kp76RmIR09NS1t",
    "NJU0ZNyz8VmU_cZHlb3fYAusPNG5V2CaLfHtXgo7DaMK_aAJwnN5hc2ri1d66hkpm8qKPpkbIY",
    "Sm0O3_lo1bwMyL3zUXjbfTQSulr43fgXdCbJ7CwINO7dTA",
);

/// Wrap JWK object literals into a JWKS document.
fn doc(entries: &[&str]) -> String {
    format!(r#"{{"keys":[{}]}}"#, entries.join(","))
}

/// A minimal RSA JWK with no `use`, `alg`, or `key_ops` restriction.
fn plain_rsa(kid: &str, modulus: &str) -> String {
    format!(r#"{{"kty":"RSA","kid":"{kid}","e":"AQAB","n":"{modulus}"}}"#)
}

/// A realistic realm document: one RS256 signing key and one encryption key.
fn two_key_jwks() -> String {
    doc(&[
        &format!(
            r#"{{"kty":"RSA","kid":"key-a","use":"sig","alg":"RS256",
                 "key_ops":["verify"],"e":"AQAB","n":"{N_A}"}}"#
        ),
        &format!(
            r#"{{"kty":"RSA","kid":"key-b","use":"enc",
                 "alg":"RS256","e":"AQAB","n":"{N_B}"}}"#
        ),
    ])
}

/// A realistic two-key document parses, keeping only the signing key.
#[test]
fn two_key_document_parses() {
    let set = JwkSet::parse(&two_key_jwks()).expect("document is well-formed");
    assert_eq!(set.keys().len(), 1, "only the `sig` key is usable");
    let key = &set.keys()[0];
    assert_eq!(key.kid.as_deref(), Some("key-a"));
    assert_eq!(key.modulus_bits, 2048);
    assert_eq!(key.modulus.len(), 256);
    assert_eq!(key.exponent, vec![0x01, 0x00, 0x01], "65537, big-endian");
    assert_eq!(key.alg, Some(SigAlg::Rs256));
    assert_eq!(key.key_ops.as_deref(), Some(&["verify".to_string()][..]));
}

/// Selection by `kid` returns that exact key and no other.
#[test]
fn selects_by_kid() {
    let body = doc(&[&plain_rsa("a", N_A), &plain_rsa("b", N_B)]);
    let set = JwkSet::parse(&body).expect("both keys are usable");
    assert_eq!(set.keys().len(), 2);
    let key = set.select(Some("b"), SigAlg::Rs256).expect("b is present");
    assert_eq!(key.kid.as_deref(), Some("b"));
    assert_eq!(key.modulus, set.keys()[1].modulus);
    assert_ne!(key.modulus, set.keys()[0].modulus, "distinct key material");
}

/// An unknown `kid` is a hard error naming the ids that were available, never a
/// silent fallback to some other key.
#[test]
fn unknown_kid_is_refused() {
    let set = JwkSet::parse(&two_key_jwks()).expect("well-formed");
    let error = set
        .select(Some("rotated-away"), SigAlg::Rs256)
        .expect_err("no such kid");
    assert!(matches!(error, JwksError::UnknownKid { .. }));
    let text = error.to_string();
    assert!(text.contains("rotated-away"), "{text}");
    assert!(text.contains("key-a"), "available ids are listed: {text}");
}

/// With no `kid` and exactly one suitable key, that key is selected.
#[test]
fn missing_kid_with_one_suitable_key_resolves() {
    let set = JwkSet::parse(&two_key_jwks()).expect("well-formed");
    let key = set
        .select(None, SigAlg::Rs256)
        .expect("exactly one candidate");
    assert_eq!(key.kid.as_deref(), Some("key-a"));
}

/// With no `kid` and several suitable keys, selection refuses rather than
/// guessing, and names every candidate.
#[test]
fn missing_kid_with_several_suitable_keys_is_ambiguous() {
    let body = doc(&[&plain_rsa("a", N_A), &plain_rsa("b", N_B)]);
    let set = JwkSet::parse(&body).expect("both usable");
    let error = set
        .select(None, SigAlg::Rs256)
        .expect_err("a tie must not be broken");
    match &error {
        JwksError::AmbiguousKey { candidates, .. } => {
            assert_eq!(candidates, &["a".to_string(), "b".to_string()]);
        }
        other => panic!("expected AmbiguousKey, got {other}"),
    }
    assert!(error.to_string().contains("refusing to guess"));
}

/// With no `kid` and nothing suitable, selection reports that plainly.
#[test]
fn missing_kid_with_no_suitable_key_is_refused() {
    let body = doc(&[&format!(
        r#"{{"kty":"RSA","kid":"a","alg":"RS512","e":"AQAB","n":"{N_A}"}}"#
    )]);
    let set = JwkSet::parse(&body).expect("usable, but only for RS512");
    let error = set
        .select(None, SigAlg::Rs256)
        .expect_err("wrong algorithm");
    assert!(matches!(error, JwksError::NoSuitableKey { .. }), "{error}");
}

/// An `enc` key is dropped at parse time, so it can never be verified against.
#[test]
fn enc_key_is_refused_for_verification() {
    let body = doc(&[&format!(
        r#"{{"kty":"RSA","kid":"enc-1","use":"enc","e":"AQAB","n":"{N_A}"}}"#
    )]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(
        set.keys().is_empty(),
        "an enc key is not a verification key"
    );
    assert_eq!(set.skipped().len(), 1);
    assert_eq!(set.skipped()[0].kid.as_deref(), Some("enc-1"));
    assert!(set.skipped()[0].reason.contains("`use` is `enc`"));
    // It is therefore unreachable by kid, too.
    assert!(set.select(Some("enc-1"), SigAlg::Rs256).is_err());
}

/// A `key_ops` list without `verify` is refused, including the `sign`-only case,
/// since `sign` is the private-key operation.
#[test]
fn key_ops_without_verify_is_refused() {
    let body = doc(&[
        &format!(r#"{{"kty":"RSA","kid":"s","key_ops":["sign"],"e":"AQAB","n":"{N_A}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"w","key_ops":["wrapKey"],"e":"AQAB","n":"{N_B}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"none","key_ops":[],"e":"AQAB","n":"{N_A}"}}"#),
    ]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert_eq!(set.skipped().len(), 3);
    for skip in set.skipped() {
        assert!(
            skip.reason.contains("does not include `verify`"),
            "{}",
            skip.reason
        );
    }
}

/// A key whose declared `alg` contradicts the request is refused, even by `kid`.
#[test]
fn alg_mismatch_is_refused() {
    let body = doc(&[&format!(
        r#"{{"kty":"RSA","kid":"a","alg":"RS512","e":"AQAB","n":"{N_A}"}}"#
    )]);
    let set = JwkSet::parse(&body).expect("usable for RS512");
    let error = set
        .select(Some("a"), SigAlg::Rs256)
        .expect_err("alg conflict");
    match &error {
        JwksError::UnsuitableKey { kid, reason } => {
            assert_eq!(kid, "a");
            assert!(reason.contains("RS512"), "{reason}");
            assert!(reason.contains("RS256"), "{reason}");
        }
        other => panic!("expected UnsuitableKey, got {other}"),
    }
    // The same key still works for the algorithm it declares.
    assert!(set.select(Some("a"), SigAlg::Rs512).is_ok());
}

/// A JWK declaring an algorithm outside the RSA-PKCS1 family is dropped, so an
/// HMAC or unsigned algorithm can never be paired with a public key.
#[test]
fn non_rsa_alg_is_skipped() {
    let body = doc(&[
        &format!(r#"{{"kty":"RSA","kid":"h","alg":"HS256","e":"AQAB","n":"{N_A}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"z","alg":"none","e":"AQAB","n":"{N_B}"}}"#),
    ]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert!(set.skipped()[0].reason.contains("HS256"));
    assert!(set.skipped()[1].reason.contains("none"));
}

/// An `oct` symmetric key is skipped, and never read as if it were RSA.
#[test]
fn oct_key_is_skipped_without_failing_the_document() {
    let body = doc(&[
        r#"{"kty":"oct","kid":"hmac","k":"c2VjcmV0LWtleS1tYXRlcmlhbA"}"#,
        &plain_rsa("good", N_A),
    ]);
    let set = JwkSet::parse(&body).expect("a mixed realm document is valid");
    assert_eq!(set.keys().len(), 1, "the RSA key survives");
    assert_eq!(set.keys()[0].kid.as_deref(), Some("good"));
    assert_eq!(set.skipped().len(), 1);
    assert!(set.skipped()[0].reason.contains("oct"));
    // The symmetric key is not reachable at all.
    assert!(set.select(Some("hmac"), SigAlg::Rs256).is_err());
    assert!(
        set.select(None, SigAlg::Rs256).is_ok(),
        "one candidate remains"
    );
}

/// An unimplemented `kty` is skipped rather than guessed at.
#[test]
fn unknown_kty_is_skipped() {
    let body = doc(&[
        r#"{"kty":"EC","kid":"ec-1","crv":"P-256","x":"AAAA","y":"AAAA"}"#,
        r#"{"kty":"OKP","kid":"ed-1","crv":"Ed25519","x":"AAAA"}"#,
        &plain_rsa("rsa-1", N_A),
    ]);
    let set = JwkSet::parse(&body).expect("document is valid");
    assert_eq!(set.keys().len(), 1);
    assert_eq!(set.skipped().len(), 2);
    assert!(set.skipped()[0].reason.contains("EC"));
    assert!(set.skipped()[1].reason.contains("OKP"));
}

/// base64url with `-`, `_`, and no padding decodes to the right big-endian bytes,
/// and the standard alphabet and explicit padding are both refused.
#[test]
fn base64url_alphabet_is_handled_exactly() {
    use tetherscript::jwks::base64url::decode;

    // 0xFB 0xFF exercises both `-` (62) and `_` (63) in a 3-character group.
    assert_eq!(decode("t", "-_8").unwrap(), vec![0xfb, 0xff]);
    // A 2-character group is one byte, with no padding present.
    assert_eq!(decode("t", "AQ").unwrap(), vec![0x01]);
    assert_eq!(decode("t", "AQAB").unwrap(), vec![0x01, 0x00, 0x01]);
    assert_eq!(decode("t", "").unwrap(), Vec::<u8>::new());

    assert!(decode("t", "+/8").unwrap_err().contains("standard base64"));
    assert!(decode("t", "-_8=").unwrap_err().contains("unpadded"));
    assert!(decode("t", "A").unwrap_err().contains("truncated"));
    assert!(decode("t", "-_.8").unwrap_err().contains("invalid"));

    // The real fixture modulus round-trips to 256 big-endian bytes.
    let modulus = decode("t", N_A).expect("fixture is valid base64url");
    assert_eq!(modulus.len(), 256);
    assert_ne!(modulus[0], 0, "minimal encoding");

    // The `-`/`_` modulus reaches the key intact, not just the decoder.
    let set = JwkSet::parse(&doc(&[&plain_rsa("a", N_A)])).expect("usable");
    assert_eq!(set.keys()[0].modulus, modulus);
}

/// A modulus with a leading zero byte is refused: RFC 7518 requires the minimal
/// big-endian encoding, and accepting both spellings splits key identity.
#[test]
fn leading_zero_modulus_is_rejected() {
    let body = doc(&[&plain_rsa("lz", LEADING_ZERO_N)]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert!(set.skipped()[0].reason.contains("leading zero"));
}

/// An even modulus cannot be a product of two odd primes, so it is refused.
#[test]
fn even_modulus_is_rejected() {
    let body = doc(&[&plain_rsa("even", EVEN_N)]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert!(
        set.skipped()[0].reason.contains("even"),
        "{}",
        set.skipped()[0].reason
    );
}

/// Exponents 0 and 1, an even exponent, and an empty one are all refused.
#[test]
fn degenerate_exponents_are_rejected() {
    // "AQ" is 0x01, "AA" is 0x00, "Ag" is 0x02, and "" is empty.
    let body = doc(&[
        &format!(r#"{{"kty":"RSA","kid":"e1","e":"AQ","n":"{N_A}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"e0","e":"AA","n":"{N_B}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"even","e":"Ag","n":"{N_A}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"empty","e":"","n":"{N_B}"}}"#),
    ]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty(), "no degenerate exponent is usable");
    assert_eq!(set.skipped().len(), 4);
    assert!(set.skipped()[0].reason.contains("must be at least 3"));
    assert!(set.skipped()[1].reason.contains("leading zero"));
    assert!(set.skipped()[2].reason.contains("even"));
    assert!(set.skipped()[3].reason.contains("empty"));
}

/// A modulus under the 2048-bit floor is refused, so a published weak key cannot
/// silently downgrade the verification chain.
#[test]
fn undersized_modulus_is_rejected() {
    let body = doc(&[&plain_rsa("weak", N_1024)]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert!(set.skipped()[0].reason.contains("1024 bits"));
    assert!(set.skipped()[0].reason.contains("2048"));
}

/// More keys than the bound allows fails the whole document, because the bound
/// caps work rather than filtering content.
#[test]
fn too_many_keys_is_rejected() {
    let one = plain_rsa("k", N_A);
    let over = vec![one.as_str(); 65];
    match JwkSet::parse(&doc(&over)).expect_err("65 exceeds the bound of 64") {
        JwksError::TooManyKeys { count, limit } => assert_eq!((count, limit), (65, 64)),
        other => panic!("expected TooManyKeys, got {other}"),
    }
    // Exactly at the bound is accepted.
    let at_limit = vec![one.as_str(); 64];
    let set = JwkSet::parse(&doc(&at_limit)).expect("64 is within the bound");
    assert_eq!(set.keys().len(), 64);
}

/// An oversized single field is refused without failing the document.
#[test]
fn oversized_field_is_rejected() {
    let huge = "A".repeat(4097);
    let body = doc(&[
        &format!(r#"{{"kty":"RSA","kid":"{huge}","e":"AQAB","n":"{N_A}"}}"#),
        &format!(r#"{{"kty":"RSA","kid":"big-n","e":"AQAB","n":"{huge}"}}"#),
    ]);
    let set = JwkSet::parse(&body).expect("document is under the size bound");
    assert!(set.keys().is_empty());
    assert_eq!(set.skipped().len(), 2);
    assert!(set.skipped()[0].reason.contains("4096"));
    assert!(set.skipped()[1].reason.contains("4096"));
}

/// An oversized document is refused before it is parsed at all.
#[test]
fn oversized_document_is_rejected() {
    let body = format!(r#"{{"keys":[],"pad":"{}"}}"#, "A".repeat(300_000));
    match JwkSet::parse(&body).expect_err("over the 256 KiB bound") {
        JwksError::DocumentTooLarge { limit, .. } => assert_eq!(limit, 262_144),
        other => panic!("expected DocumentTooLarge, got {other}"),
    }
}

/// Malformed JSON is reported as malformed JSON, with the parser's byte offset.
#[test]
fn malformed_json_is_reported_as_such() {
    let bodies = [
        "{",
        "",
        r#"{"keys":[}"#,
        r#"{"keys":[{"kty":"RSA",}]}"#,
        "not json",
    ];
    for body in bodies {
        let error = JwkSet::parse(body).expect_err("not valid JSON");
        assert!(
            matches!(error, JwksError::MalformedJson(_)),
            "{body:?} gave {error}"
        );
        assert!(error.to_string().contains("malformed JSON"));
    }
    assert!(JwkSet::parse("{").unwrap_err().to_string().contains("byte"));
}

/// Document-shape faults are distinguished from malformed JSON.
#[test]
fn document_shape_faults_are_named() {
    assert!(matches!(
        JwkSet::parse("[]").unwrap_err(),
        JwksError::NotAnObject(_)
    ));
    assert!(matches!(
        JwkSet::parse("{}").unwrap_err(),
        JwksError::MissingKeys
    ));
    assert!(matches!(
        JwkSet::parse(r#"{"keys":{}}"#).unwrap_err(),
        JwksError::KeysNotArray(_)
    ));
    // An empty keys array is valid; it simply selects nothing.
    let set = JwkSet::parse(r#"{"keys":[]}"#).expect("valid, if useless");
    assert!(matches!(
        set.select(None, SigAlg::Rs256).unwrap_err(),
        JwksError::NoSuitableKey { .. }
    ));
}

/// A non-object entry, and a missing `n`, `e`, or `kty`, are skips that name the
/// member at fault.
#[test]
fn malformed_entries_are_skipped_with_reasons() {
    let body = doc(&[
        "7",
        r#"{"kty":"RSA","kid":"no-n","e":"AQAB"}"#,
        &format!(r#"{{"kty":"RSA","kid":"no-e","n":"{N_A}"}}"#),
        r#"{"kid":"no-kty"}"#,
        &format!(r#"{{"kty":"RSA","kid":"ops","key_ops":"verify","e":"AQAB","n":"{N_A}"}}"#),
    ]);
    let set = JwkSet::parse(&body).expect("document itself is fine");
    assert!(set.keys().is_empty());
    assert_eq!(set.skipped().len(), 5);
    assert!(set.skipped()[0].reason.contains("expected a JSON object"));
    assert!(set.skipped()[1].reason.contains("`n`"));
    assert!(set.skipped()[2].reason.contains("`e`"));
    assert!(set.skipped()[3].reason.contains("`kty`"));
    assert!(set.skipped()[4].reason.contains("key_ops must be an array"));
}

/// A key with no `kid` is usable, and reachable only by unique suitability.
#[test]
fn key_without_kid_is_usable_but_not_addressable() {
    let body = doc(&[&format!(r#"{{"kty":"RSA","e":"AQAB","n":"{N_A}"}}"#)]);
    let set = JwkSet::parse(&body).expect("kid is optional");
    assert_eq!(set.keys().len(), 1);
    assert_eq!(set.keys()[0].kid, None);
    assert!(set.select(None, SigAlg::Rs256).is_ok());
    assert!(set.select(Some("anything"), SigAlg::Rs256).is_err());
}

/// A `kid` containing path-traversal characters selects normally and is never
/// interpreted. This locks in that `kid` is treated as opaque data.
#[test]
fn hostile_kid_is_opaque_data() {
    let hostile = "../../../etc/passwd";
    let body = doc(&[&plain_rsa(hostile, N_A), &plain_rsa("plain", N_B)]);
    let set = JwkSet::parse(&body).expect("a hostile kid is still just a string");
    let key = set
        .select(Some(hostile), SigAlg::Rs256)
        .expect("selects by exact match");
    assert_eq!(key.kid.as_deref(), Some(hostile));
    assert_eq!(key.modulus, set.keys()[0].modulus);
    // No normalisation happens, so a traversal-resolved spelling matches nothing.
    assert!(set.select(Some("/etc/passwd"), SigAlg::Rs256).is_err());
}

/// The `alg`, `use`, and `key_ops` members may be absent or explicitly null; both
/// spellings mean unrestricted.
#[test]
fn absent_and_null_members_are_unrestricted() {
    let body = doc(&[&format!(
        r#"{{"kty":"RSA","kid":"a","alg":null,"use":null,
             "key_ops":null,"e":"AQAB","n":"{N_A}"}}"#
    )]);
    let set = JwkSet::parse(&body).expect("nulls are read as absent");
    assert_eq!(set.keys().len(), 1);
    assert_eq!(set.keys()[0].alg, None);
    assert_eq!(set.keys()[0].key_ops, None);
    for alg in [SigAlg::Rs256, SigAlg::Rs384, SigAlg::Rs512] {
        assert!(set.select(Some("a"), alg).is_ok(), "{alg:?} is permitted");
    }
}

/// An unsupported requested algorithm is refused before any key is consulted.
#[test]
fn unsupported_requested_algorithm_is_refused() {
    for name in ["HS256", "none", "ES256", "PS256", "RS128", "rs256", ""] {
        assert!(
            matches!(
                SigAlg::parse(name).unwrap_err(),
                JwksError::UnsupportedAlgorithm(_)
            ),
            "{name} must not parse"
        );
    }
    assert_eq!(SigAlg::parse("RS256").unwrap(), SigAlg::Rs256);
    assert_eq!(SigAlg::parse("RS384").unwrap(), SigAlg::Rs384);
    assert_eq!(SigAlg::parse("RS512").unwrap(), SigAlg::Rs512);
    assert_eq!(SigAlg::Rs512.name(), "RS512");
}
