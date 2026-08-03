//! Validation of a caller-supplied `X-Request-ID`.
//!
//! # Security: an incoming request id is attacker-controlled
//!
//! A request id exists to be echoed — into response headers, into every log line
//! for the request, into a trace. That makes it the shortest path from an
//! untrusted header straight into an operator's terminal and the log store, so it
//! must be treated as hostile input rather than as an identifier.
//!
//! Two concrete attacks:
//!
//! * **Log forging.** A newline or carriage return inside the value splits one log
//!   line into two. The attacker writes the second line, so it can read as any
//!   event the format allows — an authorisation success, a different actor's
//!   address — and a later investigation believes it.
//! * **Terminal rewriting.** An ANSI escape (`ESC [ 2 J`, cursor movement, a
//!   colour reset) inside the value is interpreted when an operator `tail`s the
//!   log, so the attacker can erase or overwrite lines already on screen.
//!
//! Neither is prevented by escaping at the sink, because there are many sinks.
//! It is prevented once, here, by refusing the value.
//!
//! # The allowed charset
//!
//! ASCII alphanumerics plus hyphen and underscore: `A-Z`, `a-z`, `0-9`, `-`, `_`.
//! Length 1 to [`MAX_LEN`] characters inclusive.
//!
//! That is an allowlist, not a denylist, which is the load-bearing choice: a
//! denylist of "no newline, no ESC" would still admit the next control character,
//! a bare `\r`, a Unicode line separator (U+2028), a right-to-left override that
//! reverses how the line renders, or a NUL that truncates a C-side consumer. The
//! set above is wide enough for a UUID, a hex trace id, a W3C `traceparent`
//! segment, and a hyphenated Kubernetes-style id, and nothing in it can escape a
//! log line.

/// Longest request id echoed back, in characters.
///
/// 200 comfortably exceeds a 36-character UUID and a 55-character `traceparent`
/// while keeping a caller from appending a megabyte to every line of the log —
/// which is a cheap disk-exhaustion and log-cost attack, not merely untidy.
pub(super) const MAX_LEN: usize = 200;

/// Decide whether an incoming id may be echoed.
///
/// # Arguments
///
/// * `candidate` — The raw `X-Request-ID` value.
///
/// # Returns
///
/// `true` only when every character is ASCII alphanumeric, `-`, or `_`, and the
/// length is between 1 and [`MAX_LEN`]. Empty is rejected so a blank header
/// yields a generated id rather than an empty one that reads as "no request".
pub(super) fn is_safe(candidate: &str) -> bool {
    // `chars().count()` rather than `len()`: a multi-byte character must count
    // once, so the bound describes what an operator sees, not the encoding.
    let length = candidate.chars().count();
    if length == 0 || length > MAX_LEN {
        return false;
    }
    candidate
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}
