//! LSP range conversion, with `character` measured in UTF-16 code units.
//!
//! The Language Server Protocol's default `positionEncoding` is `utf-16`, so a
//! `character` is a UTF-16 code-unit offset into the line — not a byte offset
//! and not a `char` index. Getting this wrong is invisible in ASCII files and
//! shifts every highlight after the first emoji in files that have one.
//!
//! Unlike the terminal renderer, LSP ranges are **not** tab-expanded: the editor
//! owns tab rendering, and expanding here would move the highlight.
//!
//! A zero-width [`Span`] converts to a range whose start equals its end, which
//! LSP renders as a thin caret between characters — the right shape for
//! "expected `)` here" and end-of-file errors.

use crate::diagnostic::map::SourceMap;
use crate::diagnostic::span::Span;

/// A 0-indexed LSP position: line plus UTF-16 `character`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::LspPosition;
/// let p = LspPosition { line: 1, character: 4 };
/// assert_eq!((p.line, p.character), (1, 4));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspPosition {
    /// 0-indexed line.
    pub line: usize,
    /// 0-indexed UTF-16 code-unit offset within the line.
    pub character: usize,
}

/// A 0-indexed LSP range.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, lsp_range};
/// let map = SourceMap::new("let x = 1\n");
/// let r = lsp_range(&map, Span::new(4, 5));
/// assert_eq!(r.start.character, 4);
/// assert_eq!(r.end.character, 5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRange {
    /// Inclusive start.
    pub start: LspPosition,
    /// Exclusive end.
    pub end: LspPosition,
}

/// Converts a byte [`Span`] into an [`LspRange`].
///
/// # Arguments
///
/// * `map` — source map for the document the span belongs to.
/// * `span` — byte-offset span.
///
/// # Returns
///
/// The equivalent 0-indexed range with UTF-16 `character` offsets. Offsets past
/// the end of the buffer clamp to end of file rather than panicking, so a stale
/// span from an older document version still yields a usable range.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, lsp_range};
///
/// // "🦀" is 4 bytes, 1 char, 2 UTF-16 units. A span on the `x` after it
/// // must report character 2, not 1 and not 4.
/// let map = SourceMap::new("🦀x\n");
/// let r = lsp_range(&map, Span::new(4, 5));
/// assert_eq!(r.start, tetherscript::diagnostic::LspPosition { line: 0, character: 2 });
/// assert_eq!(r.end.character, 3);
/// ```
pub fn lsp_range(map: &SourceMap, span: Span) -> LspRange {
    let (lo, hi) = map.locate_span(span);
    LspRange {
        start: LspPosition {
            line: lo.zero_based_line(),
            character: lo.zero_based_utf16(),
        },
        end: LspPosition {
            line: hi.zero_based_line(),
            character: hi.zero_based_utf16(),
        },
    }
}
