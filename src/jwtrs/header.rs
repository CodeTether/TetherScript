//! Header parsing and the pinned-algorithm check.
//!
//! One responsibility: read `alg`, `kid`, and `typ`, and refuse any header whose
//! `alg` is not the one the *config* names.
//!
//! # Security: the verifier picks the algorithm
//!
//! This is where the two most-exploited JWT flaws are closed, and both are closed
//! by *comparison* rather than by dispatch. There is no `match alg { ... }` in this
//! module and no map from algorithm name to verification routine, so there is no
//! code path an attacker's `alg` can steer.
//!
//! * **`alg: none`** (RFC 7515 §A.5) is a legal JWS with no signature. A verifier
//!   that dispatches on `alg` reaches a "nothing to check" branch and accepts a
//!   token the attacker wrote from scratch. Here `none` is rejected by name, before
//!   the generic mismatch check, so the error says so out loud.
//! * **`alg: HS256` where `RS256` is expected** is the algorithm-confusion attack.
//!   The RSA *public* key is public. If the verifier reads `HS256` from the header
//!   and HMACs with whatever key material it looked up, the attacker can compute
//!   that HMAC too, because the secret is the published modulus. Pinning `RS256`
//!   means the token is refused before any key is fetched.
//!
//! # Security: `kid` is attacker-controlled
//!
//! [`Header::kid`] comes from an unverified segment. It is carried through as an
//! opaque lookup token for [`crate::jwtrs::verifier`] and is never interpreted
//! here. It must **never** be joined into a filesystem path, a URL path, or a
//! query: a `kid` of `../../../etc/passwd` would then turn key selection into
//! arbitrary file read. Compare it for equality against published identifiers only.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::base64url::encode;
//! use tetherscript::jwtrs::config::ValidationConfig;
//! use tetherscript::jwtrs::header::Header;
//!
//! let config = ValidationConfig::rs256("https://sso.example", ["web-app"]);
//! let good = encode(br#"{"alg":"RS256","kid":"key-a","typ":"JWT"}"#);
//! let header = Header::parse(&good, &config).unwrap();
//! assert_eq!(header.kid.as_deref(), Some("key-a"));
//!
//! // Both classic forgeries are refused against an RS256 config.
//! assert!(Header::parse(&encode(br#"{"alg":"none"}"#), &config).is_err());
//! assert!(Header::parse(&encode(br#"{"alg":"HS256"}"#), &config).is_err());
//! ```

use crate::jwtrs::config::ValidationConfig;
use crate::jwtrs::error_shape::ShapeError;
use crate::jwtrs::header_fields::{read_alg, read_optional_str};
use crate::jwtrs::segment::decode_object;

/// The JOSE header members this module cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// The algorithm the token *claims*, already confirmed equal to the config's.
    pub alg: String,
    /// Key identifier, or `None`. **Attacker-controlled**; a lookup token only.
    pub kid: Option<String>,
    /// Media type, or `None`.
    pub typ: Option<String>,
}

impl Header {
    /// Decode and check a header segment.
    ///
    /// # Arguments
    ///
    /// * `segment` — The still-encoded first segment.
    /// * `config` — Supplies the pinned algorithm and the optional `typ`
    ///   requirement.
    ///
    /// # Returns
    ///
    /// The header, with `alg` guaranteed equal to `config.algorithm`.
    ///
    /// # Errors
    ///
    /// Every [`ShapeError`] from [`decode_object`], plus
    /// [`ShapeError::MissingAlg`], [`ShapeError::AlgNotString`],
    /// [`ShapeError::AlgNone`], [`ShapeError::AlgMismatch`], and
    /// [`ShapeError::TypMismatch`].
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn parse(segment: &str, config: &ValidationConfig) -> Result<Self, ShapeError> {
        let members = decode_object("header", segment)?;
        let alg = read_alg(&members, config.algorithm)?;
        let typ = read_optional_str(&members, "typ");
        if let (Some(required), Some(found)) = (config.required_typ.as_deref(), typ.as_deref()) {
            if required != found {
                return Err(ShapeError::TypMismatch {
                    got: found.to_string(),
                    expected: required.to_string(),
                });
            }
        }
        Ok(Self {
            alg,
            kid: read_optional_str(&members, "kid"),
            typ,
        })
    }
}
