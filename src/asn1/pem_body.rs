//! Splitting the base64 body out from between the PEM armour lines.

use super::{
    error::Error,
    pem_armour::{pem_error, Armour, DASHES, END},
};

/// Slice the body between the BEGIN and END armour lines.
///
/// # Arguments
///
/// * `input` — the whole PEM text.
/// * `label` — the label read from the BEGIN line.
/// * `dashes_at` — byte offset of the closing dashes on the BEGIN line.
///
/// # Returns
///
/// An [`Armour`] whose `body` is the raw text between the two armour lines.
///
/// # Errors
///
/// [`Error::Pem`] when the END line is missing, appears before the body, or
/// carries a different label than the BEGIN line.
///
/// # Panics
///
/// Never; the body range is taken with `str::get`, so a mismatched or
/// out-of-order marker yields an error instead of an invalid slice.
pub(super) fn split<'a>(
    input: &'a str,
    label: &str,
    dashes_at: usize,
) -> Result<Armour<'a>, Error> {
    let body_start = dashes_at.saturating_add(DASHES.len());
    let end_marker = format!("{END}{label}{DASHES}");
    let end_at = input
        .find(&end_marker)
        .ok_or_else(|| pem_error(body_start, &format!("missing {end_marker} line")))?;
    if end_at < body_start {
        return Err(pem_error(end_at, "END line appears before the BEGIN line"));
    }
    let body = input
        .get(body_start..end_at)
        .ok_or_else(|| pem_error(body_start, "PEM body is not on character boundaries"))?;
    Ok(Armour {
        label: label.to_string(),
        body,
        body_offset: body_start,
    })
}
