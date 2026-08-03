//! Construction of the two OAuth requests: the authorization URL and the token body.
//!
//! # `redirect_uri` must be compared exactly, never by prefix
//!
//! An authorization server matches the `redirect_uri` in the request against the
//! values registered for the client. **That comparison must be exact.** Under prefix
//! matching, a client registered for `https://app.example.com/callback` also accepts
//! `https://app.example.com/callback.evil.example/` and
//! `https://app.example.com/callback?next=https://evil.example` — any
//! attacker-controlled *suffix* — and the authorization code is delivered to a
//! destination the attacker chose. Path-traversal suffixes such as
//! `/callback/../../open-redirect` are the same class of bug.
//!
//! This group cannot fix a badly configured server, but it enforces the client half
//! of the contract. The `redirect_uri` is required, is checked to be an absolute
//! `http(s)` URL with a host and no fragment ([`uri`]), and is read from **one** config
//! field for both the authorization request and the token request — which is what
//! guarantees the two are byte-identical. RFC 6749 §4.1.3 requires the token request
//! to echo the authorization request's value, and a difference of even a trailing
//! slash is rejected by the token endpoint.
//!
//! # No client secret in the authorization URL
//!
//! The authorization URL is a `GET` the *browser* performs, so it lands in the
//! browser's history and address bar, in the `Referer` header sent to every resource
//! the login page loads, in the access log of the authorization server and of every
//! proxy between, and in bookmarks, screen shares, and pasted support tickets. A
//! secret in any of those is a disclosed secret. [`url::build`] therefore **rejects** a
//! config containing `client_secret` rather than merely omitting it, because silently
//! dropping a field the caller supplied would hide a real configuration mistake.
//!
//! The secret belongs only in the token request: a server-to-server `POST` over TLS
//! whose body is not in a URL and not logged by default. See [`body`].
//!
//! # Mandatory `state` and `code_challenge`
//!
//! [`query::render`] reads both with `req_str`, so a caller cannot build a state-less
//! or PKCE-less authorization request even by omitting the field. Those are precisely
//! the vulnerable flows this group exists to prevent.

#[path = "oauth_request_body.rs"]
pub(crate) mod body;
#[path = "oauth_request_config.rs"]
pub(crate) mod config;
#[path = "oauth_request_query.rs"]
pub(crate) mod query;
#[path = "oauth_request_uri.rs"]
pub(crate) mod uri;
#[path = "oauth_request_url.rs"]
pub(crate) mod url;
