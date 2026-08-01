//! Recommended security response headers.
//!
//! These are defaults a handler can merge into any response. Each one closes a
//! specific, well-understood hole:
//!
//! * `X-Content-Type-Options: nosniff` — stops a browser from ignoring the
//!   declared Content-Type and guessing, which turns an uploaded text file into
//!   executable script.
//! * `X-Frame-Options: DENY` — blocks framing, defeating clickjacking. The
//!   modern equivalent is CSP `frame-ancestors`, and both are emitted because
//!   older browsers honor only the former.
//! * `Referrer-Policy: strict-origin-when-cross-origin` — sends the full URL
//!   same-origin but only the origin cross-origin, so query strings holding
//!   tokens or IDs do not leak to third parties.
//! * `Content-Security-Policy` — see [`POLICY`] for the rationale behind each
//!   directive, since a CSP copied without understanding tends to be pasted with
//!   `unsafe-inline` and quietly does nothing.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// The default Content-Security-Policy.
///
/// `default-src 'self'` denies everything not explicitly allowed, so a missed
/// resource type fails closed. `object-src 'none'` removes the plugin surface,
/// and `frame-ancestors 'none'` is the CSP form of clickjacking protection.
/// `base-uri 'self'` stops an injected `<base>` from re-pointing every relative
/// URL at an attacker.
///
/// Note there is no `unsafe-inline`: a policy that allows inline script does not
/// meaningfully mitigate XSS. A caller serving inline script must override this
/// deliberately rather than getting a weakened default by accident.
pub(super) const POLICY: &str = "default-src 'self'; object-src 'none'; \
     frame-ancestors 'none'; base-uri 'self'";

/// Build the recommended response headers.
///
/// # Returns
///
/// A map of header name to value, ready to merge into a response's `headers`.
pub(super) fn recommended() -> Value {
    let mut headers = HashMap::new();
    headers.insert(
        "x-content-type-options".to_string(),
        Value::Str(Rc::new("nosniff".to_string())),
    );
    headers.insert(
        "x-frame-options".to_string(),
        Value::Str(Rc::new("DENY".to_string())),
    );
    headers.insert(
        "referrer-policy".to_string(),
        Value::Str(Rc::new("strict-origin-when-cross-origin".to_string())),
    );
    headers.insert(
        "content-security-policy".to_string(),
        Value::Str(Rc::new(POLICY.to_string())),
    );
    Value::Map(Rc::new(RefCell::new(headers)))
}
