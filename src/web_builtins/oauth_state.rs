//! Signed, expiring OAuth `state` values that carry a return path.
//!
//! # Why state must be signed
//!
//! The `state` parameter is the only thing tying an authorization callback back to
//! the browser that started the flow. Two failures, both real:
//!
//! * **Unsigned state is a CSRF hole in the callback.** If the callback accepts any
//!   state, it cannot distinguish a state it minted from one an attacker composed.
//!   Anything derived from the state — a return path, a tenant hint, a nonce the app
//!   then trusts — becomes attacker-controlled input on an authenticated code path.
//!   An HMAC over the payload makes forgery require the secret.
//! * **Omitting state is authorization-code injection.** The attacker starts a real
//!   flow with their *own* account, captures the resulting `code` at their own
//!   redirect, then causes the victim's browser to hit
//!   `/callback?code=<attacker's code>`. With no state to check, the victim's session
//!   is silently bound to the attacker's identity, and every action the victim takes
//!   afterwards lands in the attacker's account. Requiring a state the server itself
//!   minted, in *this* browser, blocks the injected callback.
//!
//! # Why state must expire
//!
//! Without a TTL a state is valid forever, so one leaked from a browser history or a
//! proxy log stays replayable indefinitely. The expiry lives inside the signed
//! payload, so it cannot be extended without the secret.
//!
//! These tokens are stateless and therefore **not single-use**: a valid state
//! replays until it expires. A caller wanting one-time semantics must record spent
//! nonces itself, which is why a nonce is part of the payload.
//!
//! # Wire format
//!
//! The authenticated payload is five `.`-separated ASCII fields:
//!
//! ```text
//! v1 . <nonce-hex> . <issued-at> . <expires-at> . <return-to-base64url>
//! ```
//!
//! and the token is `BASE64URL(payload) "." BASE64URL(HMAC-SHA256(secret, BASE64URL(payload)))`.
//! `return_to` is base64url-encoded because the field separator is `.` and a path may
//! legitimately contain one (`/report.pdf`); encoding removes any chance of a crafted
//! path shifting the field boundaries. `v1` is a version tag so a future format change
//! is a clean rejection rather than a misparse of attacker-influenced bytes.
//!
//! # Examples
//!
//! ```tether
//! let state = oauth_state_token(secret, 600, "/dashboard")?
//! // ... on callback:
//! let destination = oauth_state_verify(secret, params.state)?
//! ```

#[path = "oauth_state_codec.rs"]
pub(crate) mod codec;
#[path = "oauth_state_mint.rs"]
pub(crate) mod mint;
#[path = "oauth_state_verify.rs"]
pub(crate) mod verify;

/// Version tag for the payload format.
pub(crate) const VERSION: &str = "v1";

/// Number of `.`-separated fields in a `v1` payload.
pub(crate) const FIELDS: usize = 5;

/// Decoded state payload.
///
/// # Examples
///
/// ```rust,ignore
/// let claims = Claims {
///     nonce: "deadbeef".into(),
///     issued_at: 1_700_000_000,
///     expires_at: 1_700_000_600,
///     return_to: "/dashboard".into(),
/// };
/// assert_eq!(claims.expires_at - claims.issued_at, 600);
/// ```
pub(crate) struct Claims {
    /// Random per-token value; exposed so callers can implement one-time use.
    pub(crate) nonce: String,
    /// Unix seconds when the state was minted.
    pub(crate) issued_at: i64,
    /// Unix seconds at and after which the state is no longer acceptable.
    pub(crate) expires_at: i64,
    /// The relative destination path, base64url-encoded while inside the payload.
    pub(crate) return_to: String,
}
