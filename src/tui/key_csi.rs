//! ANSI CSI escape sequence parsing for key events.

use super::key_event::KeyEvent;

/// Parse an ANSI CSI key sequence such as `ESC [ A`.
pub(super) fn parse(bytes: &[u8]) -> Option<KeyEvent> {
    if bytes.len() < 3 || bytes[0] != 0x1b || bytes[1] != b'[' {
        return None;
    }
    if bytes.len() == 3 {
        return named(bytes[2]);
    }
    if bytes.last() != Some(&b'~') {
        return None;
    }
    numbered(&bytes[2..bytes.len() - 1])
}

fn named(byte: u8) -> Option<KeyEvent> {
    Some(KeyEvent::named(match byte {
        b'A' => "up",
        b'B' => "down",
        b'C' => "right",
        b'D' => "left",
        b'H' => "home",
        b'F' => "end",
        _ => return None,
    }))
}

fn numbered(code: &[u8]) -> Option<KeyEvent> {
    Some(KeyEvent::named(match code {
        b"1" | b"7" => "home",
        b"2" => "insert",
        b"3" => "delete",
        b"5" => "pageup",
        b"6" => "pagedown",
        b"4" | b"8" => "end",
        _ => return None,
    }))
}
