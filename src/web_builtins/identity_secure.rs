//! Whether the request arrived over TLS.
//!
//! # Security: `X-Forwarded-Proto` is client-controlled
//!
//! `http_serve` gives a handler no transport flag, so the only in-band signal that
//! the original hop was TLS is `X-Forwarded-Proto` (or `X-Forwarded-Ssl`, or the
//! RFC 7239 `Forwarded: proto=` form). Nothing about any of them is
//! authenticated: a caller talking plaintext to a directly-exposed listener can
//! send `X-Forwarded-Proto: https` and this function will answer `true`.
//!
//! That matters because `is_secure` is exactly the condition applications use to
//! decide whether to set `Secure` cookies, whether to skip an HSTS redirect, and
//! sometimes whether to accept a credential at all. Trusting a forged value means
//! a session cookie marked `Secure` is issued over a channel that is not, and the
//! redirect that would have upgraded the connection never fires.
//!
//! **It is trustworthy only behind a reverse proxy that overwrites the header on
//! every inbound request** — not one that appends to it, and not one that passes a
//! client-supplied value through. This is the same trust condition
//! `header_client_ip` documents for `X-Forwarded-For`, and it fails the same way:
//! honoured on a directly-exposed listener, it lets the caller decide the answer.
//! When the deployment cannot guarantee such a proxy, terminate TLS in-process and
//! ignore this field.

use std::collections::HashMap;

use super::identity_headers::find;
use crate::value::Value;

/// Decide whether the original request hop was TLS.
///
/// # Arguments
///
/// * `headers` — Header map from the request.
///
/// # Returns
///
/// `true` when `X-Forwarded-Proto` is `https` (case-insensitively, taking the
/// leftmost entry of a comma list because each proxy appends and the leftmost is
/// the original client hop), or when `X-Forwarded-Ssl` is `on`. `false` when the
/// header is absent — failing closed, so a missing proxy header is treated as
/// plaintext rather than optimistically as TLS.
pub(super) fn is_secure(headers: &HashMap<String, Value>) -> bool {
    if let Some(proto) = find(headers, "x-forwarded-proto") {
        let first = proto.split(',').next().unwrap_or("").trim();
        return first.eq_ignore_ascii_case("https");
    }
    find(headers, "x-forwarded-ssl").is_some_and(|flag| flag.eq_ignore_ascii_case("on"))
}
