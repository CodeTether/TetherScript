//! Reversible escaping for the session-store wire format.
//!
//! # Why a naive split loses data
//!
//! The serialized form is `key=value` entries joined by `;`. A session payload
//! routinely holds text an application never inspected — a display name, a return
//! URL with a query string, a flash message with a newline. If such a value were
//! written raw, `text.split(';')` would tear one entry into two and
//! `split_once('=')` would truncate a URL at its first `=`. That is silent
//! corruption: the decode succeeds and returns wrong data.
//!
//! # The escaping
//!
//! Five characters are escaped, backslash-first so the transform is injective:
//!
//! | Raw | Escaped | Why |
//! |---|---|---|
//! | `\` | `\\` | the escape character itself, or unescaping is ambiguous |
//! | `;` | `\s` | entry separator |
//! | `=` | `\e` | key/value separator |
//! | LF | `\n` | line-oriented logs and RESP inline framing |
//! | CR | `\r` | same, and `\r\n` must not be splittable |
//!
//! After escaping, no separator byte remains in any component, so a plain split is
//! then safe. Nothing else is touched: UTF-8 passes through unchanged.

/// Entry separator.
pub(super) const ENTRY_SEP: char = ';';
/// Key/value separator within an entry.
pub(super) const PAIR_SEP: char = '=';

/// Escape one component so it contains no separator.
///
/// # Arguments
///
/// * `raw` — Arbitrary UTF-8 text.
///
/// # Returns
///
/// Text free of `\`, `;`, `=`, LF, and CR.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(escape("a;b=c"), "a\\sb\\ec");
/// ```
pub(super) fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ENTRY_SEP => out.push_str("\\s"),
            PAIR_SEP => out.push_str("\\e"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out
}
