//! The slug charset and length limit.
//!
//! # Charset
//!
//! A valid slug is a non-empty run of ASCII lowercase letters, ASCII digits,
//! `-`, and `_`. Nothing else. That is an allowlist, and an allowlist is the only
//! form of this check that stays correct as the set of dangerous characters
//! grows: a denylist has to enumerate `..`, `/`, `\`, NUL, `%`, `:` (Windows ADS
//! and drive letters), and whatever the next filesystem or template engine
//! invents. The allowlist rejects all of those without naming any of them, so
//! `..` is already impossible here — `.` is simply not a member.
//!
//! Uppercase is excluded because the slug is *normalised* to lowercase before
//! validation ([`super::dynpage_slug`]): allowing both would make `About` and
//! `about` two cache keys for one page, which halves the hit rate and doubles the
//! chance of a stale entry.
//!
//! # Length limit
//!
//! `MAX_LEN` is **200 bytes**. The limit exists because the slug may become part
//! of a filesystem path, and most filesystems cap a single component at 255
//! bytes; 200 leaves room for a prefix directory and an extension a caller adds
//! after this check. It is a byte limit, not a character count, since the
//! filesystem counts bytes — and because the charset is ASCII-only, the two
//! coincide for anything that passes.

/// Longest accepted slug, in bytes. See the module docs for the derivation.
pub(super) const MAX_LEN: usize = 200;

/// Test one byte for membership in the slug charset.
///
/// # Arguments
///
/// * `byte` — Candidate byte.
///
/// # Returns
///
/// True when the byte is ASCII lowercase, an ASCII digit, `-`, or `_`.
fn allowed(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
}

/// Test a whole slug against the charset and the length limit.
///
/// # Arguments
///
/// * `slug` — Candidate slug, already normalised.
///
/// # Returns
///
/// True when the slug is non-empty, within [`MAX_LEN`] bytes, and made entirely
/// of allowed bytes.
pub(super) fn valid(slug: &str) -> bool {
    !slug.is_empty() && slug.len() <= MAX_LEN && slug.bytes().all(allowed)
}
