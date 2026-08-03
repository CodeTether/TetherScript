//! [`SourceMap`] — precomputed line index over one source buffer.
//!
//! ## Complexity
//!
//! Construction scans the buffer once: **O(n)** time in the byte length, and
//! **O(L)** extra space for `L` lines (one `usize` per line start). Resolving
//! one offset to a line is a binary search over the line-start table:
//! **O(log L)**. Computing the columns then walks only the bytes of that single
//! line, so a full [`SourceMap::locate`] is `O(log L + line_len)`.
//!
//! The naive alternative — rescanning the whole file per diagnostic — is
//! `O(n)` each, so a file with `d` diagnostics costs `O(n·d)`, i.e. quadratic
//! when the error count grows with file size (a lexer that reports every bad
//! character does exactly that). The map makes it `O(n + d·log L)`.
//!
//! ## Examples
//!
//! ```rust
//! use tetherscript::diagnostic::SourceMap;
//!
//! let map = SourceMap::with_name("demo.tether", "let a = 1\nlet b = 2\n");
//! assert_eq!(map.name(), "demo.tether");
//! assert_eq!(map.line_count(), 3); // trailing newline opens an empty line 3
//! assert_eq!(map.line_text(2), "let b = 2");
//! assert_eq!(map.line_start(2), 10);
//! ```

/// An immutable source buffer plus its line-start index.
///
/// Construct one per file (or per LSP document version) and reuse it for every
/// diagnostic on that file.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::SourceMap;
/// let map = SourceMap::new("a\nbb\n");
/// assert_eq!(map.line_text(1), "a");
/// assert_eq!(map.line_text(2), "bb");
/// assert_eq!(map.line_text(3), "");
/// ```
#[derive(Debug, Clone)]
pub struct SourceMap {
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) line_starts: Vec<usize>,
}

impl SourceMap {
    /// Builds a map for anonymous text (REPL input, tests).
    ///
    /// # Arguments
    ///
    /// * `text` — the complete source buffer.
    ///
    /// # Returns
    ///
    /// A `SourceMap` named `<anon>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("x").name(), "<anon>");
    /// ```
    pub fn new(text: &str) -> Self {
        Self::with_name("<anon>", text)
    }

    /// Builds a map for a named file.
    ///
    /// # Arguments
    ///
    /// * `name` — path or URI shown in the rendered header.
    /// * `text` — the complete source buffer.
    ///
    /// # Returns
    ///
    /// A `SourceMap` whose line-start table has already been computed in one
    /// `O(n)` pass.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// let map = SourceMap::with_name("main.tether", "fn main() {}\n");
    /// assert_eq!(map.name(), "main.tether");
    /// assert_eq!(map.line_count(), 2);
    /// ```
    pub fn with_name(name: &str, text: &str) -> Self {
        let mut line_starts = vec![0usize];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self {
            name: name.to_string(),
            text: text.to_string(),
            line_starts,
        }
    }

    /// The file name used in rendered headers.
    ///
    /// # Returns
    ///
    /// The name given at construction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::with_name("a.tether", "").name(), "a.tether");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The whole source buffer.
    ///
    /// # Returns
    ///
    /// The text this map indexes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("hi").text(), "hi");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Number of lines, counting a trailing empty line after a final newline.
    ///
    /// # Returns
    ///
    /// The size of the line-start table, always at least 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("").line_count(), 1);
    /// assert_eq!(SourceMap::new("a\n").line_count(), 2);
    /// assert_eq!(SourceMap::new("a\nb").line_count(), 2);
    /// ```
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte offset where a 1-indexed line begins.
    ///
    /// # Arguments
    ///
    /// * `line` — 1-indexed line number; out-of-range values clamp to the last
    ///   line so rendering can never panic on a stale span.
    ///
    /// # Returns
    ///
    /// The line's start byte offset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// let map = SourceMap::new("ab\ncd\n");
    /// assert_eq!(map.line_start(1), 0);
    /// assert_eq!(map.line_start(2), 3);
    /// assert_eq!(map.line_start(999), 6);
    /// ```
    pub fn line_start(&self, line: usize) -> usize {
        let idx = line.saturating_sub(1).min(self.line_starts.len() - 1);
        self.line_starts[idx]
    }

    /// Text of a 1-indexed line, without its line terminator.
    ///
    /// # Arguments
    ///
    /// * `line` — 1-indexed line number; out-of-range clamps to the last line.
    ///
    /// # Returns
    ///
    /// The line contents with any trailing `\n` and `\r` removed, so rendering
    /// a CRLF file does not emit a stray carriage return into the terminal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::SourceMap;
    /// assert_eq!(SourceMap::new("ab\r\ncd").line_text(1), "ab");
    /// ```
    pub fn line_text(&self, line: usize) -> &str {
        let start = self.line_start(line);
        let end = match self.line_starts.get(line) {
            Some(next) => *next,
            None => self.text.len(),
        };
        self.text[start..end]
            .trim_end_matches('\n')
            .trim_end_matches('\r')
    }
}
