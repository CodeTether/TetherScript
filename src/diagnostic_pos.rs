//! [`LineCol`] — the presentation-side coordinate produced from a byte offset.
//!
//! One offset has three defensible columns, so we compute all three once and
//! let each consumer pick:
//!
//! * `byte_col` — bytes into the line. Useful for slicing the line again.
//! * `char_col` — Unicode scalar values. What a human terminal message means.
//! * `utf16_col` — UTF-16 code units. What LSP means.
//!
//! All three are **1-indexed** because that is what error messages print; LSP's
//! 0-indexed form is produced by the explicit accessors below so the conversion
//! is never accidental.

/// A resolved source position: one line, three column flavours.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{LineCol, SourceMap};
///
/// let map = SourceMap::new("é🦀x\n");
/// let at_x = map.locate("é🦀".len());
/// assert_eq!(at_x.line, 1);
/// assert_eq!(at_x.byte_col, 7);   // 2 + 4 bytes + 1
/// assert_eq!(at_x.char_col, 3);   // two chars before it
/// assert_eq!(at_x.utf16_col, 4);  // the crab is a surrogate pair
/// assert_eq!(at_x.zero_based_utf16(), 3);
/// assert_eq!(at_x.zero_based_line(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed byte column within the line.
    pub byte_col: usize,
    /// 1-indexed `char` (Unicode scalar value) column within the line.
    pub char_col: usize,
    /// 1-indexed UTF-16 code-unit column within the line.
    pub utf16_col: usize,
}

impl LineCol {
    /// The 0-indexed line, as LSP wants it.
    ///
    /// # Returns
    ///
    /// `line - 1`, saturating at zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("a\nb").locate(2).zero_based_line(), 1);
    /// ```
    pub fn zero_based_line(&self) -> usize {
        self.line.saturating_sub(1)
    }

    /// The 0-indexed UTF-16 column, as LSP's `character` field wants it.
    ///
    /// # Returns
    ///
    /// `utf16_col - 1`, saturating at zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("ab").locate(1).zero_based_utf16(), 1);
    /// ```
    pub fn zero_based_utf16(&self) -> usize {
        self.utf16_col.saturating_sub(1)
    }
}

impl std::fmt::Display for LineCol {
    /// Formats as `line:char_col`, the terminal convention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("ab").locate(1).to_string(), "1:2");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.char_col)
    }
}
