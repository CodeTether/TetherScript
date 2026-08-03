//! The trait boundary between claim validation and RSA/JWKS machinery.
//!
//! One responsibility: state, as a trait, exactly what this module needs from the
//! outside world — "verify these signing-input bytes against the key selected by
//! this `kid`" — and nothing more.
//!
//! # Why a trait and not a direct call
//!
//! `src/rsa/verify.rs` and `src/jwks/keyset.rs` exist in this tree but are not
//! declared in `src/lib.rs` yet, so they are not callable. More importantly, claim
//! validation *should not* depend on them: it needs a yes/no on a signature, not a
//! modulus. The trait is therefore the permanent design, not a workaround.
//!
//! It keeps three things out of this module: RSA arithmetic, JWKS parsing, and
//! HTTP. All three are somebody else's file.
//!
//! # How the integrator wires the real modules to this
//!
//! Write one adapter that owns a parsed `JwkSet` and implements this trait:
//!
//! ```rust,ignore
//! impl SignatureVerifier for KeycloakVerifier {
//!     fn verify(&self, kid: Option<&str>, alg: &str, signing_input: &[u8], signature: &[u8])
//!         -> Result<(), String>
//!     {
//!         // 1. `alg` is already pinned by the caller; map it to a SigAlg.
//!         let sig_alg = SigAlg::parse(alg).map_err(|e| e.to_string())?;
//!         // 2. Select the key. `kid` is attacker-controlled; `select` treats an
//!         //    unknown `kid` as a hard error, never a fallback.
//!         let key = self.keys.select(kid, sig_alg).map_err(|e| e.to_string())?;
//!         // 3. Hash the signing input, then check the PKCS#1 v1.5 signature.
//!         let digest = sha256(signing_input);
//!         rsa::verify(signature, &digest, DigestAlgorithm::Sha256, &key.into())
//!             .map_err(|e| e.to_string())
//!     }
//! }
//! ```
//!
//! # Security: `kid` is attacker-controlled
//!
//! `kid` arrives in the *unverified* header. It is a lookup key among keys the
//! issuer already published, and that is its only legitimate use. An implementor
//! **must never** interpolate `kid` into a filesystem path, a URL path, an SQL
//! fragment, or a shell word: `kid` of `../../etc/shadow` or `%2e%2e%2f` would then
//! turn key selection into arbitrary file read. Compare it for equality against
//! already-known key identifiers and nothing else. Likewise an unknown `kid` must
//! be an error, not a fall back to "try every key", which turns selection into key
//! roulette.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::jwtrs::verifier::SignatureVerifier;
//! use tetherscript::jwtrs::test_verifier::StubVerifier;
//!
//! let verifier = StubVerifier::accepting("sig-ok");
//! assert!(verifier.verify(Some("key-a"), "RS256", b"anything", b"sig-ok").is_ok());
//! assert!(verifier.verify(Some("key-a"), "RS256", b"anything", b"forged").is_err());
//! ```

/// Something that can decide whether a JWS signature is authentic.
///
/// Implemented by the RSA/JWKS adapter in production and by
/// [`StubVerifier`](crate::jwtrs::test_verifier::StubVerifier) in tests.
pub trait SignatureVerifier {
    /// Verify one signature.
    ///
    /// # Arguments
    ///
    /// * `kid` — The header's `kid`, or `None` when absent. **Attacker-controlled**;
    ///   use it only as an equality lookup among published keys.
    /// * `alg` — The algorithm the *caller* has already pinned, never the token's
    ///   own claim. Passed so an implementor can select an algorithm-scoped key.
    /// * `signing_input` — Exactly the ASCII bytes
    ///   `header_b64 || "." || payload_b64`, taken from the original token text so
    ///   no re-encoding can change them.
    /// * `signature` — The decoded third segment.
    ///
    /// # Returns
    ///
    /// `Ok(())` if and only if `signature` is authentic for `signing_input`.
    ///
    /// # Errors
    ///
    /// Returns a message naming the failed step — unknown `kid`, unsuitable key,
    /// bad signature length, or a failed comparison. The caller wraps it in
    /// [`JwtError::Signature`](crate::jwtrs::error::JwtError::Signature) without
    /// interpreting it.
    fn verify(
        &self,
        kid: Option<&str>,
        alg: &str,
        signing_input: &[u8],
        signature: &[u8],
    ) -> Result<(), String>;
}
