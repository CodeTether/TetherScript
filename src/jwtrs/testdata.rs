//! Token fixtures for doc examples and tests.
//!
//! One responsibility: assemble a compact serialization from literal header and
//! payload JSON, so every doc example in `crate::jwtrs` is *runnable* rather than
//! `ignore`d.
//!
//! # Why this ships in the library
//!
//! A doc example cannot reach into a `tests/` file, and this repository's
//! documentation rules require runnable examples wherever possible. The helpers
//! here perform no cryptography — the "signature" is a literal byte string checked
//! by [`StubVerifier`](crate::jwtrs::test_verifier::StubVerifier) — so nothing here
//! weakens the real path.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::testdata::{keycloak_token, token_with};
//!
//! let token = token_with(r#"{"alg":"RS256"}"#, r#"{"sub":"u"}"#, "sig-ok");
//! assert_eq!(token.split('.').count(), 3);
//! assert!(keycloak_token(1_000, "sig-ok").contains('.'));
//! ```

use crate::jwtrs::base64url::encode;

/// Assemble a token from literal JSON and a literal signature.
///
/// # Arguments
///
/// * `header` — Header JSON, base64url-encoded verbatim, including deliberately
///   malformed input.
/// * `payload` — Payload JSON, likewise verbatim.
/// * `signature` — Bytes placed in the third segment.
///
/// # Returns
///
/// The compact serialization `header.payload.signature`.
///
/// # Panics
///
/// Does not panic.
pub fn token_with(header: &str, payload: &str, signature: &str) -> String {
    format!(
        "{}.{}.{}",
        encode(header.as_bytes()),
        encode(payload.as_bytes()),
        encode(signature.as_bytes())
    )
}

/// A Keycloak-shaped RS256 token with realm and resource roles.
///
/// Issuer is `https://sso.example/realms/main`, audience `web-app`, subject
/// `user-1`, `azp` `web-app`, `nbf` and `iat` at `exp - 300`.
///
/// # Arguments
///
/// * `exp` — The expiry, in seconds since the Unix epoch.
/// * `signature` — Bytes for the third segment.
///
/// # Returns
///
/// The compact serialization.
///
/// # Panics
///
/// Does not panic.
pub fn keycloak_token(exp: i64, signature: &str) -> String {
    let issued = exp - 300;
    let payload = format!(
        concat!(
            r#"{{"iss":"https://sso.example/realms/main","sub":"user-1","aud":"web-app","#,
            r#""exp":{exp},"nbf":{issued},"iat":{issued},"azp":"web-app","#,
            r#""realm_access":{{"roles":["admin","offline_access"]}},"#,
            r#""resource_access":{{"web-app":{{"roles":["viewer"]}}}}}}"#
        ),
        // Named explicitly rather than captured implicitly: implicit format-argument
        // capture does not see through a `concat!`-built format string.
        exp = exp,
        issued = issued,
    );
    token_with(
        r#"{"alg":"RS256","kid":"key-a","typ":"JWT"}"#,
        &payload,
        signature,
    )
}
