//! What the cursor is sitting on.
//!
//! Both hover and definition need the same three facts: the identifier under
//! the cursor, its byte range, and whether it is qualified by a `.` (as in
//! `math.add` or `text.trim()`). The VSCode client derived these with
//! `getWordRangeAtPosition` plus a regex over the line text; this module
//! derives them from the document bytes so the server does not depend on any
//! editor API.
//!
//! Word characters are ASCII alphanumerics and `_`, matching
//! [`crate::lexer::Lexer`]'s identifier rule exactly. A non-ASCII character is
//! therefore *not* part of a word, which is the correct behaviour and also the
//! reason the UTF-16 conversion in
//! [`crate::lsp_capabilities::position`] must happen before this module runs.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::context::word_at;
//!
//! let word = word_at("let total = 1", 5).expect("cursor is inside a word");
//! assert_eq!(word.text, "total");
//! assert_eq!((word.start, word.end), (4, 9));
//! assert!(word.qualifier.is_none());
//! ```

/// The identifier under a cursor, with its byte range and `.` qualifier.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::context::word_at;
/// let word = word_at("math.add(1)", 6).expect("inside `add`");
/// assert_eq!(word.text, "add");
/// assert_eq!(word.qualifier.as_deref(), Some("math"));
/// ```
#[derive(Debug, Clone)]
pub struct Word {
    /// The identifier text.
    pub text: String,
    /// Byte offset of the first byte of the identifier.
    pub start: usize,
    /// Byte offset just past the identifier.
    pub end: usize,
    /// Identifier immediately before a `.` preceding this word, if any.
    pub qualifier: Option<String>,
}

/// Identify the word touching byte `offset`.
///
/// A cursor immediately after a word counts as inside it, matching editor
/// behaviour: hovering with the caret at `total|` should still describe `total`.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `offset` — Cursor byte offset, clamped to the document length.
///
/// # Returns
///
/// `Some(Word)` when a word touches the cursor, `None` on whitespace or
/// punctuation. `None` is not an error — it means "nothing to describe" — and
/// callers reply with a null result.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::context::word_at;
/// // Whitespace on both sides: nothing to describe.
/// assert!(word_at("a  b", 2).is_none());
/// // A cursor immediately after a word still names that word.
/// assert_eq!(word_at("let x = 1", 3).map(|word| word.text), Some("let".into()));
/// assert_eq!(word_at("let x = 1", 9).map(|word| word.text), Some("1".into()));
/// assert!(word_at("", 0).is_none());
/// ```
pub fn word_at(text: &str, offset: usize) -> Option<Word> {
    let bytes = text.as_bytes();
    let offset = offset.min(bytes.len());
    let mut start = offset;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_word_byte(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    Some(Word {
        text: text[start..end].to_string(),
        start,
        end,
        qualifier: qualifier_before(text, start),
    })
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The identifier that qualifies a word starting at `start`, if any.
///
/// # Arguments
///
/// * `text` — Full document text.
/// * `start` — Byte offset of the qualified word's first byte.
///
/// # Returns
///
/// `Some(owner)` when the bytes before `start` are `owner.`, else `None`.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::context::qualifier_before;
/// assert_eq!(qualifier_before("math.add", 5).as_deref(), Some("math"));
/// assert_eq!(qualifier_before("add", 0), None);
/// ```
pub fn qualifier_before(text: &str, start: usize) -> Option<String> {
    let bytes = text.as_bytes();
    if start == 0 || bytes.get(start - 1) != Some(&b'.') {
        return None;
    }
    let owner_end = start - 1;
    let mut owner_start = owner_end;
    while owner_start > 0 && is_word_byte(bytes[owner_start - 1]) {
        owner_start -= 1;
    }
    if owner_start == owner_end {
        return None;
    }
    Some(text[owner_start..owner_end].to_string())
}
