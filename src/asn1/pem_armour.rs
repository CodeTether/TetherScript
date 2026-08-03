//! Locating the BEGIN/END armour lines of a PEM block.
//!
//! Splitting armour location from base64 decoding keeps each file to one
//! concern: this one only answers "where is the body, and what is it called".

use super::error::Error;

/// The `-----BEGIN ` prefix.
pub(super) const BEGIN: &str = "-----BEGIN ";
/// The `-----END ` prefix.
pub(super) const END: &str = "-----END ";
/// The five-dash run that closes both armour lines.
pub(super) const DASHES: &str = "-----";

/// The located armour: label, body text, and the body's byte offset.
pub(super) struct Armour<'a> {
    pub(super) label: String,
    pub(super) body: &'a str,
    pub(super) body_offset: usize,
}

/// Build a PEM error at `offset` with `reason`.
///
/// # Arguments
///
/// * `offset` — byte offset within the PEM text.
/// * `reason` — human-readable cause.
///
/// # Returns
///
/// An [`Error::Pem`] value.
pub(super) fn pem_error(offset: usize, reason: &str) -> Error {
    Error::Pem {
        offset,
        reason: reason.to_string(),
    }
}

/// Find the first PEM block's armour and body.
///
/// # Arguments
///
/// * `input` — text that should contain a PEM block.
///
/// # Returns
///
/// An [`Armour`] borrowing the body text out of `input`.
///
/// # Errors
///
/// [`Error::Pem`] when a marker is missing or the BEGIN and END labels differ.
///
/// # Panics
///
/// Never; every slice index comes from a `str::find` match or `str::get`, and
/// the `get` fallback path returns an error rather than panicking.
pub(super) fn locate(input: &str) -> Result<Armour<'_>, Error> {
    let begin = input
        .find(BEGIN)
        .ok_or_else(|| pem_error(0, "missing -----BEGIN <LABEL>----- line"))?;
    let label_start = begin.saturating_add(BEGIN.len());
    let rest = input
        .get(label_start..)
        .ok_or_else(|| pem_error(label_start, "BEGIN line is truncated"))?;
    let label_len = rest
        .find(DASHES)
        .ok_or_else(|| pem_error(label_start, "BEGIN line has no closing dashes"))?;
    let label = rest
        .get(..label_len)
        .ok_or_else(|| pem_error(label_start, "BEGIN label is not a character boundary"))?;
    super::pem_body::split(input, label, label_start.saturating_add(label_len))
}
