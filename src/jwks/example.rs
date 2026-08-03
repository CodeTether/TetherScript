//! A hand-written two-key JWKS used by the doc examples.
//!
//! One responsibility: hold the fixture. It lives in the library rather than only
//! in the tests so every doc example in this module is *runnable* against real key
//! material instead of pseudocode.
//!
//! # Provenance
//!
//! Both moduli are the real 2048-bit moduli of RSA keys generated with
//! `openssl genrsa 2048`, base64url-encoded and unpadded. `AQAB` is the standard
//! public exponent 65537. `key-a` is a signing key; `key-b` is an encryption key,
//! present so the examples can show it being refused for verification.

/// A two-key JWKS document: one RS256 signing key and one encryption key.
///
/// # Examples
///
/// ```rust
/// use tetherscript::jwks::keyset::{EXAMPLE_JWKS, JwkSet};
///
/// let set = JwkSet::parse(EXAMPLE_JWKS).unwrap();
/// // Both entries are well-formed RSA keys, but only the signing one is kept.
/// assert_eq!(set.keys().len(), 1);
/// assert_eq!(set.keys()[0].kid.as_deref(), Some("key-a"));
/// assert!(set.skipped()[0].reason.contains("`use` is `enc`"));
/// ```
pub const EXAMPLE_JWKS: &str = concat!(
    r#"{"keys":[{"kty":"RSA","kid":"key-a","use":"sig","alg":"RS256","#,
    r#""key_ops":["verify"],"e":"AQAB","n":""#,
    "q9HPapksy45yL75Z_RcMu_f8pPnreongJsZAlGrkeH8nETCOGdJ8wTy08MFsdVsvRjRFfUbpPGVX",
    "CtTS5xSfBh_k5vc5mQPmBLJQOZ21w-T3TxCc4t8CO-MwGF8oe-VIDrnNSfimFu-T50d1Coelbn6_",
    "yq-eJuQQBYqPHW9du9h336oPFaEmLvcfRpmkxuIkiwjX0HzEvxf_eqf3kp76RmIR09NS1tNJU0ZN",
    "yz8VmU_cZHlb3fYAusPNG5V2CaLfHtXgo7DaMK_aAJwnN5hc2ri1d66hkpm8qKPpkbIYSm0O3_lo",
    "1bwMyL3zUXjbfTQSulr43fgXdCbJ7CwINO7dTQ",
    r#""},"#,
    r#"{"kty":"RSA","kid":"key-b","use":"enc","alg":"RS256","e":"AQAB","n":""#,
    "nweI2q5sFK2_Cg71kNSHBARlDq720TxRSo9dZF9N5okxWt4_HWUwyTcBr-CEFgxaLkj_eFqWyHb0",
    "y6EDEpmrCidhlSEqRjQ0LEs3-pjp-GlPaNwFfv501gWmd9mNtp2litE1kY_xvVd8u9XHvf_Xf134",
    "-UKL5sTAe2zcNLcmghXEfsWYQLH6-33S2MR5C5vtlRmni4_bpXW--wlF8gh5sp4Fp4d9PMNKgMDC",
    "YGEEIXyJfogQ1KklPNVK9IVgSZBpsPvN80eh-ZuCvDL0z5qZCIMZ69AQYIw6bnfXVgsLyi5UE53k",
    "7zgN0Fo8RGNOtc_Hh0BLz-LOmAsAXTALNo9J0w",
    r#""}]}"#,
);
