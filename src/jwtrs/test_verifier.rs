//! A self-contained [`SignatureVerifier`] test double.
//!
//! One responsibility: decide signature acceptance by exact byte comparison
//! against a fixed expected signature, so every claim-validation behaviour can be
//! exercised without RSA key material.
//!
//! # Why this lives in `src/` and not only in `tests/`
//!
//! The doc examples throughout `crate::jwtrs` must be **runnable**, per this
//! repository's documentation rules. A doc example cannot reach into a `tests/`
//! file, so the double ships in the library. It is inert: it performs no
//! cryptography and grants nothing.
//!
//! # This is not a weakening of the real path
//!
//! [`StubVerifier`] is only ever reachable by code that constructs one. Nothing in
//! [`crate::jwtrs::validate`] has a default verifier, so no production caller can
//! end up using this by omission — the verifier is a required argument.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::test_verifier::StubVerifier;
//! use tetherscript::jwtrs::verifier::SignatureVerifier;
//!
//! let good = StubVerifier::accepting("sig-ok");
//! assert!(good.verify(None, "RS256", b"input", b"sig-ok").is_ok());
//!
//! let never = StubVerifier::rejecting();
//! assert!(never.verify(None, "RS256", b"input", b"sig-ok").is_err());
//! ```

use crate::jwtrs::verifier::SignatureVerifier;

/// A verifier that accepts exactly one signature byte string, or nothing at all.
#[derive(Debug, Clone)]
pub struct StubVerifier {
    /// The only signature accepted; `None` means reject everything.
    expected: Option<Vec<u8>>,
}

impl StubVerifier {
    /// Build a double that accepts exactly `signature` and nothing else.
    ///
    /// # Arguments
    ///
    /// * `signature` — The decoded signature bytes to treat as authentic.
    ///
    /// # Returns
    ///
    /// The configured double.
    pub fn accepting(signature: &str) -> Self {
        Self {
            expected: Some(signature.as_bytes().to_vec()),
        }
    }

    /// Build a double that rejects every signature.
    ///
    /// # Returns
    ///
    /// The configured double, used to assert that no claim survives a signature
    /// failure.
    pub fn rejecting() -> Self {
        Self { expected: None }
    }
}

impl SignatureVerifier for StubVerifier {
    /// # Errors
    ///
    /// Returns a message when `signature` differs from the configured one, or when
    /// the double was built with [`StubVerifier::rejecting`].
    fn verify(
        &self,
        _kid: Option<&str>,
        _alg: &str,
        _signing_input: &[u8],
        signature: &[u8],
    ) -> Result<(), String> {
        match &self.expected {
            Some(expected) if expected == signature => Ok(()),
            Some(_) => Err("stub: signature does not match the expected bytes".to_string()),
            None => Err("stub: configured to reject every signature".to_string()),
        }
    }
}
