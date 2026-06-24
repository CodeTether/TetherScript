//! Decode JSON messages carried by stdio.

use crate::{json, value::Value};

pub(super) fn line(line: &str) -> Result<Value, String> {
    parse(
        line.trim_end_matches(['\r', '\n'])
            .trim_start_matches('\u{feff}'),
    )
}

pub(super) fn body(body: &[u8]) -> Result<Value, String> {
    let text = std::str::from_utf8(body)
        .map_err(|error| format!("stdio_read: body is not UTF-8: {error}"))?;
    parse(text.trim_start_matches('\u{feff}'))
}

fn parse(text: &str) -> Result<Value, String> {
    json::parse_str(text).map_err(|error| format!("stdio_read: {error}"))
}
