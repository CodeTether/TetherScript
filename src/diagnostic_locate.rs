//! Offset → [`LineCol`] resolution on [`SourceMap`].
//!
//! Split out of [`crate::diagnostic::map`] so that indexing (construction) and
//! querying (resolution) stay one responsibility each.
//!
//! The line lookup is a binary search over the line-start table — `O(log L)` —
//! and the column walk touches only the bytes of the single line the offset
//! falls on. See [`crate::diagnostic::map`] for the full complexity argument.

use crate::diagnostic::pos::LineCol;
use crate::diagnostic::span::Span;
use crate::diagnostic::utf16::{char_len, utf16_len};

impl crate::diagnostic::SourceMap {
    /// Resolves a byte offset to a 1-indexed line and all three column flavours.
    ///
    /// # Arguments
    ///
    /// * `offset` — byte offset into the source. Offsets past the end clamp to
    ///   the end of the buffer, so an end-of-file span resolves instead of
    ///   panicking. An offset that is not a `char` boundary is floored to the
    ///   boundary at or before it.
    ///
    /// # Returns
    ///
    /// A [`LineCol`] whose `byte_col`, `char_col` and `utf16_col` all describe
    /// the same position measured in bytes, `char`s and UTF-16 code units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    ///
    /// let map = SourceMap::new("let x = 1\nlet y = 2\n");
    /// let at_y = map.locate(14);
    /// assert_eq!(at_y.line, 2);
    /// assert_eq!(at_y.char_col, 5);
    ///
    /// // Past the end clamps to EOF rather than panicking.
    /// let eof = map.locate(9_999);
    /// assert_eq!(eof.line, 3);
    /// assert_eq!(eof.char_col, 1);
    /// ```
    pub fn locate(&self, offset: usize) -> LineCol {
        let offset = self.floor_boundary(offset.min(self.text.len()));
        let idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let start = self.line_starts[idx];
        let prefix = &self.text[start..offset];
        LineCol {
            line: idx + 1,
            byte_col: prefix.len() + 1,
            char_col: char_len(prefix) + 1,
            utf16_col: utf16_len(prefix) + 1,
        }
    }

    /// Resolves both ends of a span.
    ///
    /// # Arguments
    ///
    /// * `span` — the span to resolve.
    ///
    /// # Returns
    ///
    /// `(start, end)` as [`LineCol`] pairs. For a zero-width span the two are
    /// equal, which is the correct LSP representation of a caret between
    /// characters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{SourceMap, Span};
    ///
    /// let map = SourceMap::new("ab\ncd\n");
    /// let (lo, hi) = map.locate_span(Span::new(1, 4));
    /// assert_eq!((lo.line, lo.char_col), (1, 2));
    /// assert_eq!((hi.line, hi.char_col), (2, 2));
    /// ```
    pub fn locate_span(&self, span: Span) -> (LineCol, LineCol) {
        (self.locate(span.start), self.locate(span.end))
    }

    /// Largest `char` boundary at or before `offset`.
    fn floor_boundary(&self, offset: usize) -> usize {
        let mut o = offset;
        while o > 0 && !self.text.is_char_boundary(o) {
            o -= 1;
        }
        o
    }
}
