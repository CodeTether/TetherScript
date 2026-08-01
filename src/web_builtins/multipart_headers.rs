//! Per-part header parsing: `Content-Disposition` and `Content-Type`.
//!
//! A part's headers are separated from its body by CRLF CRLF, and each header is
//! `Name: value`. Only the two fields the port consumes are surfaced, matching
//! how the reference reads `get_name` and `get_filename` off Content-Disposition.

use super::multipart_boundary::{split_unquoted, unquote};

/// The `name`, `filename`, and `content_type` declared by one part.
#[derive(Default)]
pub(super) struct Headers {
    pub(super) name: Option<String>,
    pub(super) filename: Option<String>,
    pub(super) content_type: Option<String>,
}

/// Parse a part's header block.
///
/// # Arguments
///
/// * `block` — The raw header text, without the trailing CRLF CRLF.
///
/// # Returns
///
/// The recognized fields. Unknown headers are ignored rather than rejected, since
/// browsers legitimately add `Content-Transfer-Encoding` and others.
pub(super) fn parse(block: &str) -> Headers {
    let mut headers = Headers::default();
    // A header value may itself be folded across CRLF; splitting on CRLF alone is
    // sufficient here because form-data parts do not fold in practice.
    for line in block.split("\r\n") {
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        match field.trim().to_ascii_lowercase().as_str() {
            "content-disposition" => disposition(value, &mut headers),
            "content-type" => headers.content_type = Some(value.trim().to_string()),
            _ => {}
        }
    }
    headers
}

/// Read `name` and `filename` from a `Content-Disposition` value.
fn disposition(value: &str, headers: &mut Headers) {
    for part in split_unquoted(value).into_iter().skip(1) {
        let Some((key, raw)) = part.split_once('=') else {
            continue;
        };
        match key.trim().to_ascii_lowercase().as_str() {
            "name" => headers.name = Some(unquote(raw.trim())),
            "filename" => headers.filename = Some(unquote(raw.trim())),
            _ => {}
        }
    }
}
