//! The verify-before-trust gate, made structural.
//!
//! One responsibility: own the *only* way to obtain a decoded payload, and make
//! that way run signature verification first.
//!
//! # How the ordering is structural rather than commented
//!
//! Three properties together make "read claims before checking the signature"
//! unrepresentable rather than merely discouraged:
//!
//! 1. **[`Authenticated`] has a private field.** No code outside this file can
//!    construct one, so no code outside this file can hold a decoded payload.
//! 2. **The only constructor is [`Authenticated::verify`],** which takes a
//!    [`SignatureVerifier`] as a *required* argument — no default, no `Option`,
//!    no "skip" flag — and returns `Err` before touching the payload segment.
//! 3. **The payload is not even decoded until after verification.** The `?` on the
//!    verifier call sits *above* the [`decode_object`] call, so on a refused
//!    signature the payload bytes are never base64url-decoded, never parsed as
//!    JSON, and never allocated as claims. There is no partially-trusted
//!    intermediate value for a later refactor to accidentally hand back.
//!
//! Every accessor in [`crate::jwtrs::claims`] takes `&Authenticated`, so *holding*
//! one is the proof that a verifier accepted the signature. A caller cannot ask for
//! `sub` without first having produced that proof.
//!
//! Verification itself needs only the header and the signing input, and the header's
//! contribution is limited to a pinned-algorithm comparison plus an opaque `kid`
//! lookup token — so nothing semantic is trusted ahead of the signature either.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::authenticated::Authenticated;
//! use tetherscript::jwtrs::config::ValidationConfig;
//! use tetherscript::jwtrs::test_verifier::StubVerifier;
//! use tetherscript::jwtrs::testdata::token_with;
//!
//! let config = ValidationConfig::rs256("https://sso.example", ["web-app"]);
//! let token = token_with(r#"{"alg":"RS256"}"#, r#"{"sub":"user-1"}"#, "sig-ok");
//!
//! // Right signature: an `Authenticated` exists, so the payload may be read.
//! assert!(Authenticated::verify(&token, &config, &StubVerifier::accepting("sig-ok")).is_ok());
//! // Wrong signature: no `Authenticated`, so no claim is reachable at all.
//! assert!(Authenticated::verify(&token, &config, &StubVerifier::rejecting()).is_err());
//! ```

use std::collections::HashMap;

use crate::jwtrs::compact::Parts;
use crate::jwtrs::config::ValidationConfig;
use crate::jwtrs::error::JwtError;
use crate::jwtrs::header::Header;
use crate::jwtrs::segment::decode_object;
use crate::jwtrs::signature::decode_signature;
use crate::jwtrs::verifier::SignatureVerifier;
use crate::value::Value;

/// A payload whose signature a verifier has already accepted.
///
/// The `payload` field is private on purpose; read it through
/// [`crate::jwtrs::claims`].
#[derive(Debug, Clone)]
pub struct Authenticated {
    /// The checked header, kept so callers can log the `kid` that was selected.
    pub header: Header,
    payload: HashMap<String, Value>,
}

impl Authenticated {
    /// Split, pin the algorithm, verify the signature, *then* decode the payload.
    ///
    /// # Arguments
    ///
    /// * `token` — The compact serialization, without any `Bearer ` prefix.
    /// * `config` — Supplies the pinned algorithm and the `typ` requirement.
    /// * `verifier` — Decides signature authenticity; see [`SignatureVerifier`].
    ///
    /// # Returns
    ///
    /// An `Authenticated` whose payload is known to be a JSON object.
    ///
    /// # Errors
    ///
    /// [`JwtError::Shape`] for size, segment-count, base64url, UTF-8, JSON, and
    /// algorithm failures, and [`JwtError::Signature`] when the verifier refuses.
    /// No [`JwtError::Claim`] can originate here.
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn verify(
        token: &str,
        config: &ValidationConfig,
        verifier: &dyn SignatureVerifier,
    ) -> Result<Self, JwtError> {
        let parts = Parts::split(token)?;
        let header = Header::parse(parts.header_b64, config)?;
        let signature = decode_signature(parts.signature_b64)?;
        verifier
            .verify(
                header.kid.as_deref(),
                config.algorithm,
                parts.signing_input.as_bytes(),
                &signature,
            )
            .map_err(JwtError::Signature)?;
        // Nothing above this line has looked at the payload segment at all.
        let payload = decode_object("payload", parts.payload_b64)?;
        Ok(Self { header, payload })
    }

    /// Borrow the authenticated payload members.
    ///
    /// # Returns
    ///
    /// The decoded JSON object. Reachable only through an `Authenticated`, so the
    /// borrow is itself evidence that the signature was checked.
    pub(crate) fn payload(&self) -> &HashMap<String, Value> {
        &self.payload
    }
}
