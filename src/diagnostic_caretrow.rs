//! Building the caret row that sits under a rendered source line.
//!
//! The row is produced in the *expanded cell space* described in
//! [`crate::diagnostic::caret`]: leading padding is the display width of the
//! text before the span, and the caret run is the display width of the span
//! itself.
//!
//! Two edge cases have defined behaviour:
//!
//! * **Zero-width span** — rendered as exactly one `^`, so the reader still
//!   gets a location. Underlining nothing would be invisible.
//! * **Span continuing past the line** — the caret run stops at end of line and
//!   ` ...` is appended, marking that the span continues on later lines.

use crate::diagnostic::caret::{display_width, display_width_at};

/// Builds the caret row for a byte range inside one source line.
///
/// # Arguments
///
/// * `line` — the source line, without its terminator, tabs *not* expanded.
/// * `start` — byte offset of the span start within `line`, clamped to the line.
/// * `end` — byte offset of the span end within `line`, clamped to the line.
/// * `continues` — whether the span extends beyond this line.
///
/// # Returns
///
/// A string of spaces followed by at least one `^`, aligned against
/// [`crate::diagnostic::caret::expand_tabs`] of the same line, plus a trailing
/// ` ...` when `continues` is set.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::caretrow::caret_row;
///
/// assert_eq!(caret_row("let x = 1", 4, 5, false), "    ^");
/// // Zero width still shows one caret.
/// assert_eq!(caret_row("let x = 1", 4, 4, false), "    ^");
/// // Tabs are accounted for: one tab is four cells.
/// assert_eq!(caret_row("\tlet", 1, 4, false), "    ^^^");
/// // A span leaving the line is marked.
/// assert_eq!(caret_row("ab", 0, 2, true), "^^ ...");
/// ```
pub fn caret_row(line: &str, start: usize, end: usize, continues: bool) -> String {
    let (a, b) = clamp(line, start, end);
    let pad = display_width(&line[..a]);
    let run = display_width_at(&line[a..b], pad).max(1);
    let mut out = " ".repeat(pad);
    out.extend(std::iter::repeat_n('^', run));
    if continues {
        out.push_str(" ...");
    }
    out
}

/// Clamps a byte range into `line` and onto `char` boundaries.
fn clamp(line: &str, start: usize, end: usize) -> (usize, usize) {
    let mut a = start.min(line.len());
    let mut b = end.clamp(a, line.len());
    while a > 0 && !line.is_char_boundary(a) {
        a -= 1;
    }
    while b < line.len() && !line.is_char_boundary(b) {
        b += 1;
    }
    (a, b)
}
