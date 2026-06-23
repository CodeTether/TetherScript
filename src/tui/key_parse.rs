//! Terminal byte parsing for key events.

use super::{key_csi, key_event::KeyEvent};

/// Parse the first key event from bytes read from stdin.
pub(super) fn parse(bytes: &[u8]) -> Result<KeyEvent, String> {
    if bytes.is_empty() {
        return Err("tui_read_key: no input bytes".into());
    }
    if let Some(event) = key_csi::parse(bytes) {
        return Ok(event);
    }
    match bytes[0] {
        b'\r' | b'\n' => Ok(KeyEvent::named("enter")),
        b'\t' => Ok(KeyEvent::named("tab")),
        0x08 | 0x7f => Ok(KeyEvent::named("backspace")),
        0x1b if bytes.len() == 1 => Ok(KeyEvent::named("escape")),
        0x1b if bytes[1] == b'[' => Err(sequence_error(bytes)),
        0x1b => parse(&bytes[1..]).map(KeyEvent::alt),
        1..=26 => Ok(KeyEvent::ctrl((b'a' + bytes[0] - 1) as char)),
        0x20..=0x7e => Ok(KeyEvent::text(bytes[0] as char)),
        _ => parse_utf8(bytes),
    }
}

fn parse_utf8(bytes: &[u8]) -> Result<KeyEvent, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        format!(
            "tui_read_key: invalid utf-8 input byte 0x{:02x}: {error}",
            bytes[0]
        )
    })?;
    text.chars()
        .next()
        .map(KeyEvent::text)
        .ok_or_else(|| "tui_read_key: empty utf-8 input".into())
}

fn sequence_error(bytes: &[u8]) -> String {
    let parts = bytes
        .iter()
        .map(|byte| format!("0x{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("tui_read_key: unsupported escape sequence {parts}")
}
