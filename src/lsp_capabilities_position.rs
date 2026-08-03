//! LSP position ↔ byte offset conversion.
//!
//! The Language Server Protocol specifies a `Position` as a zero-based line
//! number plus a zero-based `character` offset counted in **UTF-16 code
//! units**. TetherScript's lexer, by contrast, reports 1-based line/column
//! pairs where the column counts **bytes**, and every symbol table in this
//! module stores plain byte offsets into the document text.
//!
//! Those three coordinate systems agree for pure ASCII and disagree the moment
//! a document contains one non-ASCII character, which is why an
//! ASCII-only test suite happily "passes" while every completion and jump in a
//! real file lands one or more columns to the left. Conversion therefore lives
//! here, in one place, and is exercised directly by
//! `tests/lsp_capabilities.rs` with a multi-byte character before the cursor.
//!
//! ## Clamping policy
//!
//! LSP clients may legitimately send a `character` past the end of a line (for
//! example when a keystroke races the document sync). The spec says to clamp
//! to the line end, so [`byte_offset`] clamps rather than failing. A line
//! number that does not exist at all is a genuine mismatch between client and
//! server state, so it yields `None` and the caller replies with a null result
//! instead of panicking.

/// Byte offset of the first character of a zero-based `line`.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `line` — Zero-based line number, as sent by an LSP client.
///
/// # Returns
///
/// `Some(offset)` when the line exists, `None` when the document has fewer
/// lines than requested.
///
/// # Errors
///
/// Infallible; out-of-range input is reported as `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::position::line_start;
///
/// assert_eq!(line_start("a\nbb\n", 0), Some(0));
/// assert_eq!(line_start("a\nbb\n", 1), Some(2));
/// assert_eq!(line_start("a\nbb\n", 9), None);
/// ```
pub fn line_start(text: &str, line: usize) -> Option<usize> {
    if line == 0 {
        return Some(0);
    }
    let mut seen = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == line {
                return Some(index + 1);
            }
        }
    }
    None
}

/// Convert an LSP position into a byte offset into `text`.
///
/// `character` is interpreted as a count of UTF-16 code units from the start
/// of the line, so a leading `"é"` (one UTF-16 unit, two UTF-8 bytes) advances
/// `character` by one and the returned offset by two.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `line` — Zero-based line number.
/// * `character` — Zero-based UTF-16 code-unit offset within the line.
///
/// # Returns
///
/// `Some(byte_offset)`, clamped to the end of the line when `character` runs
/// past it, or `None` when `line` does not exist.
///
/// # Errors
///
/// Infallible; invalid input is reported as `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::position::byte_offset;
///
/// // "é" is 2 bytes but 1 UTF-16 unit, so 5 units land at byte 6.
/// assert_eq!(byte_offset("let é = 1", 0, 5), Some(6));
/// // 4 units land at the first byte of "é" itself.
/// assert_eq!(byte_offset("let é = 1", 0, 4), Some(4));
/// // Past the line end clamps instead of failing.
/// assert_eq!(byte_offset("ab\ncd", 0, 99), Some(2));
/// assert_eq!(byte_offset("ab", 7, 0), None);
/// ```
pub fn byte_offset(text: &str, line: usize, character: usize) -> Option<usize> {
    let start = line_start(text, line)?;
    let mut units = 0usize;
    for (index, ch) in text[start..].char_indices() {
        if units >= character || ch == '\n' {
            return Some(start + index);
        }
        units += ch.len_utf16();
    }
    Some(text.len())
}

/// Convert a byte offset into an LSP `(line, character)` pair.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Byte offset; clamped to the document and floored to the
///   nearest character boundary so a mid-character offset cannot panic.
///
/// # Returns
///
/// A zero-based line number and a zero-based UTF-16 code-unit column.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::position::offset_position;
///
/// assert_eq!(offset_position("ab\ncd", 4), (1, 1));
/// // The "é" before the cursor counts as one UTF-16 unit, not two bytes.
/// assert_eq!(offset_position("é!", 3), (0, 2));
/// assert_eq!(offset_position("é!", 900), (0, 2));
/// ```
pub fn offset_position(text: &str, offset: usize) -> (usize, usize) {
    let offset = floor_boundary(text, offset);
    let before = &text[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let start = before.rfind('\n').map(|index| index + 1).unwrap_or(0);
    (line, before[start..].chars().map(char::len_utf16).sum())
}

fn floor_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}
