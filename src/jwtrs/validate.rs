//! The end-to-end validation entry point.
//!
//! One responsibility: run the stages in order and hand back typed claims. This is
//! the only function most callers need.
//!
//! # The order, and why it is this order
//!
//! ```text
//! 1. size / segment count / base64url / JSON shape   untrusted bytes
//! 2. pinned `alg` comparison                          untrusted bytes
//! 3. SIGNATURE VERIFICATION                           <- the trust boundary
//! 4. claim extraction                                 authenticated
//! 5. issuer, audience, time window                    authenticated
//! ```
//!
//! Steps 1–3 live inside [`Authenticated::verify`], and steps 4–5 can only run on
//! its output, because [`Claims`] has no other constructor. So the boundary is
//! enforced by the type system rather than by the order of statements below: even a
//! careless rewrite of this function cannot read a claim before the signature is
//! checked, because there is nothing to read it from.
//!
//! # Claims are never returned on failure
//!
//! Every error path returns `Err(JwtError)`. `JwtError` carries no claim payload —
//! only names, and the concrete values of the *failed* check. A caller that ignores
//! the `Err` receives no `Claims` at all, so "log the subject of the rejected token"
//! is not something this API can be tricked into.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::claims::Claims;
//! use tetherscript::jwtrs::config::ValidationConfig;
//! use tetherscript::jwtrs::test_verifier::StubVerifier;
//! use tetherscript::jwtrs::testdata::keycloak_token;
//!
//! let config = ValidationConfig::rs256("https://sso.example/realms/main", ["web-app"]);
//! let verifier = StubVerifier::accepting("sig-ok");
//! let token = keycloak_token(1_000, "sig-ok");
//!
//! assert!(Claims::validate(&token, &config, 950, &verifier).is_ok());
//! // Well past `exp`, even allowing the 60s default skew.
//! assert!(Claims::validate(&token, &config, 5_000, &verifier).is_err());
//! // A different service's audience.
//! let other = ValidationConfig::rs256("https://sso.example/realms/main", ["billing"]);
//! assert!(Claims::validate(&token, &other, 950, &verifier).is_err());
//! ```

use crate::jwtrs::audience::matches_any;
use crate::jwtrs::authenticated::Authenticated;
use crate::jwtrs::claims::Claims;
use crate::jwtrs::config::ValidationConfig;
use crate::jwtrs::error::JwtError;
use crate::jwtrs::error_claims::ClaimError;
use crate::jwtrs::time_window::check;
use crate::jwtrs::verifier::SignatureVerifier;

impl Claims {
    /// Verify a token's signature and validate its claims.
    ///
    /// # Arguments
    ///
    /// * `token` — The compact serialization, without any `Bearer ` prefix.
    /// * `config` — The verifier's own expectations; see [`ValidationConfig`].
    /// * `now` — Seconds since the Unix epoch. Injected rather than read from the
    ///   clock; see [`crate::jwtrs::time_window`] for the three reasons.
    /// * `verifier` — Decides signature authenticity; see [`SignatureVerifier`].
    ///
    /// # Returns
    ///
    /// The validated [`Claims`] on success, and nothing at all otherwise.
    ///
    /// # Errors
    ///
    /// [`JwtError::Shape`] for a malformed token or an `alg` that is not the
    /// configured one, [`JwtError::Signature`] when the verifier refuses, and
    /// [`JwtError::Claim`] for a missing or wrong-typed claim, an issuer or audience
    /// mismatch, or a token outside its validity window.
    ///
    /// # Panics
    ///
    /// Does not panic.
    pub fn validate(
        token: &str,
        config: &ValidationConfig,
        now: i64,
        verifier: &dyn SignatureVerifier,
    ) -> Result<Self, JwtError> {
        let authenticated = Authenticated::verify(token, config, verifier)?;
        check(authenticated.payload(), now, config.skew_secs)?;
        let claims = Self::extract(&authenticated)?;
        if claims.iss != config.issuer {
            return Err(ClaimError::IssuerMismatch {
                got: claims.iss,
                expected: config.issuer.clone(),
            }
            .into());
        }
        if !matches_any(&claims.aud, &config.audiences) {
            return Err(ClaimError::AudienceMismatch {
                got: claims.aud,
                expected: config.audiences.clone(),
            }
            .into());
        }
        Ok(claims)
    }
}
