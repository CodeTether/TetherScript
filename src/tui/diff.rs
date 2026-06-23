//! Redraw helpers for terminal frames.

/// Compose a stable full-screen redraw sequence.
pub(super) fn full_redraw(frame: &str) -> String {
    let mut out = String::with_capacity(frame.len() + 24);
    out.push_str("\x1b[?25l\x1b[2J\x1b[H");
    out.push_str(frame);
    out.push_str("\x1b[?25h");
    out
}
