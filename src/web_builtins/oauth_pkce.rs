//! PKCE code verifiers and S256 challenges (RFC 7636).
//!
//! # Why PKCE at all
//!
//! In the plain authorization-code flow, whoever holds the code can redeem it if
//! they also hold the client secret. Public clients — a single-page app, a mobile
//! app, a CLI — cannot keep a secret, so the code alone is enough. PKCE binds the
//! code to a per-request secret the client keeps in memory: the authorization
//! request carries only `SHA-256(verifier)`, and the token request must present the
//! verifier itself.
//!
//! # Why S256, and never `plain`
//!
//! RFC 7636 also defines `method=plain`, where the challenge *is* the verifier.
//! That offers **no protection against an intercepted authorization code**: an
//! attacker who can read the redirect — a shared device's history, a `Referer`
//! leak, a hostile app registered on the same custom URI scheme — reads the
//! verifier straight out of the same authorization request and redeems the code.
//! S256 sends only the digest, which is not invertible, so the interceptor learns
//! nothing usable. This group implements S256 only; there is no way to ask for
//! `plain`.
//!
//! # Verifier shape
//!
//! RFC 7636 §4.1 fixes the verifier at 43 to 128 characters from the unreserved
//! set `ALPHA / DIGIT / "-" / "." / "_" / "~"`. The lower bound is about entropy:
//! 43 base64url characters carry the 256 bits §7.1 asks for. The upper bound and
//! the alphabet are about interoperability — a longer or differently-encoded
//! verifier is rejected by conforming servers, and a character needing
//! percent-encoding invites a mismatch between what the client hashed and what the
//! server received.
//!
//! [`gen::generate`] produces exactly 43 characters from 32 bytes of OS entropy.
//! [`gen::challenge`] accepts any verifier in the legal range, so a caller minting
//! its own value still gets a validated challenge.

#[path = "oauth_pkce_gen.rs"]
pub(crate) mod gen;
#[path = "oauth_pkce_pair.rs"]
pub(crate) mod pair;

/// Minimum verifier length in characters (RFC 7636 §4.1).
pub(crate) const MIN_VERIFIER: usize = 43;

/// Maximum verifier length in characters (RFC 7636 §4.1).
pub(crate) const MAX_VERIFIER: usize = 128;

/// The only challenge method this group supports.
pub(crate) const METHOD: &str = "S256";

/// True when `byte` is in the RFC 3986 unreserved set.
///
/// # Arguments
///
/// * `byte` — A single ASCII byte.
///
/// # Returns
///
/// True for `A-Z`, `a-z`, `0-9`, `-`, `.`, `_`, and `~`; false for everything
/// else, including every non-ASCII byte.
pub(crate) fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}
