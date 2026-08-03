//! Line/column → byte offset, the inverse of [`crate::diagnostic::locate`].
//!
//! Needed because editors and LSP clients speak positions, not offsets: an
//! incoming `textDocument/definition` position must become a [`Span`] before it
//! can be compared against spans recorded by the lexer.
//!
//! Complexity is `O(1)` for the line lookup (direct index into the line-start
//! table) plus a walk of that one line's bytes.

use crate::diagnostic::span::Span;
use crate::diagnostic::utf16::byte_offset_for_utf16;

impl crate::diagnostic::SourceMap {
    /// Converts a 1-indexed line and 1-indexed UTF-16 column to a byte offset.
    ///
    /// # Arguments
    ///
    /// * `line` — 1-indexed line number; clamped to the last line.
    /// * `utf16_col` — 1-indexed UTF-16 code-unit column; clamped to the end of
    ///   the line, and floored to a `char` boundary if it names the low half of
    ///   a surrogate pair.
    ///
    /// # Returns
    ///
    /// The corresponding byte offset into the source.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    ///
    /// let map = SourceMap::new("let a = 1\n🦀 = 2\n");
    /// assert_eq!(map.offset_of(1, 5), 4);
    /// // The crab is two UTF-16 units, so column 3 is the space after it.
    /// assert_eq!(map.offset_of(2, 3), 10 + 4);
    /// ```
    pub fn offset_of(&self, line: usize, utf16_col: usize) -> usize {
        let start = self.line_start(line);
        let text = self.line_text(line);
        start + byte_offset_for_utf16(text, utf16_col.saturating_sub(1))
    }

    /// Converts an LSP-style 0-indexed range to a [`Span`].
    ///
    /// # Arguments
    ///
    /// * `start` — `(line, character)` both 0-indexed, `character` in UTF-16
    ///   code units.
    /// * `end` — `(line, character)`, same convention.
    ///
    /// # Returns
    ///
    /// The byte-offset [`Span`] covering that range.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{SourceMap, Span};
    ///
    /// let map = SourceMap::new("abc\ndef\n");
    /// assert_eq!(map.span_of_lsp((0, 1), (1, 2)), Span::new(1, 6));
    /// ```
    pub fn span_of_lsp(&self, start: (usize, usize), end: (usize, usize)) -> Span {
        Span::new(
            self.offset_of(start.0 + 1, start.1 + 1),
            self.offset_of(end.0 + 1, end.1 + 1),
        )
    }
}
