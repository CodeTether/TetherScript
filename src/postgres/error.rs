//! `ErrorResponse` and `NoticeResponse` field decoding.
//!
//! The body is a sequence of one-byte field tags, each followed by a
//! NUL-terminated string, terminated by a zero byte. Severity, SQLSTATE, and the
//! primary message are surfaced so failures name what went wrong.

use super::cursor::Cursor;

/// Build a human-readable error string from an `ErrorResponse` body.
pub(super) fn describe(body: &[u8]) -> String {
    let mut cursor = Cursor::new(body);
    let mut severity = String::new();
    let mut code = String::new();
    let mut message = String::new();
    let mut detail = String::new();

    while let Ok(tag) = cursor.take(1) {
        if tag[0] == 0 {
            break;
        }
        let Ok(text) = cursor.cstr() else { break };
        match tag[0] {
            b'S' => severity = text,
            b'C' => code = text,
            b'M' => message = text,
            b'D' => detail = text,
            _ => {}
        }
    }

    if message.is_empty() {
        return "postgres: server reported an error with no message field".into();
    }
    let mut out = format!("postgres: {severity}: {message}");
    if !code.is_empty() {
        out.push_str(&format!(" (SQLSTATE {code})"));
    }
    if !detail.is_empty() {
        out.push_str(&format!(" — {detail}"));
    }
    out
}
