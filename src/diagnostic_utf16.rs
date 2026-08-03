//! UTF-16 code-unit arithmetic.
//!
//! The Language Server Protocol measures a `character` in **UTF-16 code
//! units**, not bytes and not Unicode scalar values. For every character in the
//! Basic Multilingual Plane the two agree, which is exactly why the bug hides
//! in ASCII-only testing; for anything outside the BMP (emoji, astral-plane
//! CJK, mathematical alphanumerics) one `char` is *two* UTF-16 code units and a
//! character-count column silently under-reports.
//!
//! These helpers are deliberately dependency-free: `char::len_utf16` is in
//! `core`.

/// Counts the UTF-16 code units in `text`.
///
/// # Arguments
///
/// * `text` — any UTF-8 string slice, typically the part of a line preceding a
///   position.
///
/// # Returns
///
/// The number of UTF-16 code units. Surrogate pairs count as two.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::utf16;
///
/// assert_eq!(utf16::utf16_len("abc"), 3);
/// assert_eq!(utf16::utf16_len("é"), 1); // BMP: 2 bytes, 1 char, 1 unit
/// assert_eq!(utf16::utf16_len("🦀"), 2); // astral: 4 bytes, 1 char, 2 units
/// ```
pub fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

/// Counts the Unicode scalar values (`char`s) in `text`.
///
/// # Arguments
///
/// * `text` — any UTF-8 string slice.
///
/// # Returns
///
/// The `char` count, which is what a human means by "column".
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::utf16;
/// assert_eq!(utf16::char_len("🦀é"), 2);
/// ```
pub fn char_len(text: &str) -> usize {
    text.chars().count()
}

/// Inverse of [`utf16_len`]: converts a UTF-16 column back to a byte offset.
///
/// # Arguments
///
/// * `text` — the line (or prefix) the column is measured in.
/// * `units` — a 0-based UTF-16 code-unit count.
///
/// # Returns
///
/// The byte offset in `text` reached after `units` code units. If `units`
/// lands *inside* a surrogate pair the offset of the whole character is
/// returned (we never produce a non-char-boundary offset). If `units` exceeds
/// the line, `text.len()` is returned.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::utf16;
///
/// let line = "🦀x";
/// assert_eq!(utf16::byte_offset_for_utf16(line, 0), 0);
/// assert_eq!(utf16::byte_offset_for_utf16(line, 1), 0); // inside the pair
/// assert_eq!(utf16::byte_offset_for_utf16(line, 2), 4); // after the crab
/// assert_eq!(utf16::byte_offset_for_utf16(line, 99), line.len());
/// ```
pub fn byte_offset_for_utf16(text: &str, units: usize) -> usize {
    let mut seen = 0usize;
    for (offset, ch) in text.char_indices() {
        if seen >= units {
            return offset;
        }
        let next = seen + ch.len_utf16();
        if next > units {
            return offset;
        }
        seen = next;
    }
    text.len()
}
