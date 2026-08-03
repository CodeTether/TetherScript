//! The session address-change signal.
//!
//! The reference application pairs `IdentityMiddleware` with a tracker that records
//! the address a session was created from and compares it on each request. This is
//! the comparison, and nothing more: the caller owns the session map, exactly as
//! `ratelimit` leaves the caller owning the bucket, so there is no global mutable
//! state and the check is directly testable.
//!
//! # Security: this is a signal, not a verdict
//!
//! An address change is weak evidence of session theft, and it must not be wired to
//! a hard failure. Legitimate rotation is routine:
//!
//! * Mobile networks re-assign a carrier NAT address mid-session, and a handset
//!   moving between cell sites or between Wi-Fi and LTE changes address in seconds.
//! * Corporate egress pools and CGNAT rotate the visible address per connection.
//! * IPv6 privacy extensions rotate the host portion on a timer, by design.
//! * VPN and Tor users change exit nodes constantly.
//!
//! Hard-failing on any of those logs a user out mid-form. Worse, it teaches the
//! operator to disable the check, so the one real theft is not caught either.
//!
//! The right responses are: record it in the audit trail with both addresses;
//! re-authenticate before a privileged action; raise the weight of other signals.
//! The wrong response is to terminate the session — and the wrong response is also
//! to *trust* a stable address as proof of continuity, since an attacker behind the
//! same NAT looks identical.
//!
//! Because the address itself may come from `X-Forwarded-For`, it is only as
//! trustworthy as the proxy in front of it; see `header_client_ip`.

use crate::value::Value;

/// Session fields consulted for the recorded address, in order.
///
/// `client_ip` matches the field name `request_context` produces, so a session
/// created from a context needs no renaming; `created_ip` is the name the reference
/// tracker used and is accepted for compatibility.
const RECORDED_FIELDS: [&str; 2] = ["client_ip", "created_ip"];

/// Whether the caller's address differs from the session's recorded one.
///
/// # Arguments
///
/// * `session` — The session context map.
/// * `current` — The address of the request now being served.
///
/// # Returns
///
/// `true` when both addresses are known, non-empty, and unequal. `false` when they
/// match, and `false` when either is absent or empty — an unknown address is not
/// evidence of a change, and reporting one would make every session with no
/// recorded address look stolen on its first request.
///
/// Comparison is exact string equality on the textual form, so it reports a change
/// for `::1` against `0:0:0:0:0:0:0:1`, which are the same host. Acceptable
/// precisely because the result is a signal to log rather than a verdict: a false
/// positive costs a log line, and normalising would need an IP parser this group has
/// no other use for.
pub(super) fn changed(session: &Value, current: &str) -> bool {
    let Value::Map(map) = session else {
        return false;
    };
    let map = map.borrow();
    let recorded = RECORDED_FIELDS
        .iter()
        .find_map(|field| match map.get(*field) {
            Some(Value::Str(text)) if !text.is_empty() => Some((**text).clone()),
            _ => None,
        });
    match recorded {
        Some(original) if !current.is_empty() => original != current,
        _ => false,
    }
}
