//! One snippet block: the `-->` locator, the source line(s), and the carets.
//!
//! ## Multi-line spans
//!
//! A span crossing a line boundary is rendered as every line it touches: the
//! first line gets a caret run from the span start to end of line marked
//! ` ...`, intermediate lines are shown as context without carets, and the last
//! line gets a caret run from its start to the span end and carries the label.
//! Both endpoints stay visible without drawing box art.
//!
//! ## Zero-width and end-of-file spans
//!
//! A zero-width span renders a single `^` between characters. At end of file the
//! resolved line is the (possibly empty) final line, so the block still has a
//! source row — an empty one — and a caret under column 1.

use crate::diagnostic::caretrow::caret_row;
use crate::diagnostic::gutter::{bar, header, src_row};
use crate::diagnostic::map::SourceMap;
use crate::diagnostic::span::Span;

/// Renders one snippet block for `span`.
///
/// # Arguments
///
/// * `map` — source map for the file the span belongs to.
/// * `span` — the span to underline.
/// * `label` — inline message printed after the final caret run, or `None`.
/// * `gutter` — width of the line-number field, from
///   [`crate::diagnostic::render::gutter_width`].
///
/// # Returns
///
/// The block's rows, each without a trailing newline.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, snippet::block};
///
/// let map = SourceMap::with_name("a.tether", "let x = 1\n");
/// let rows = block(&map, Span::new(4, 5), Some("here"), 1);
/// assert_eq!(
///     rows,
///     vec![
///         " --> a.tether:1:5".to_string(),
///         "  |".to_string(),
///         "1 | let x = 1".to_string(),
///         "  |     ^ here".to_string(),
///         "  |".to_string(),
///     ]
/// );
/// ```
pub fn block(map: &SourceMap, span: Span, label: Option<&str>, gutter: usize) -> Vec<String> {
    let (lo, hi) = map.locate_span(span);
    let mut out = vec![header(gutter, map.name(), lo), bar(gutter)];
    for line in lo.line..=hi.line {
        out.push(src_row(gutter, line, map.line_text(line)));
        push_marks(&mut out, map, span, (line, lo.line, hi.line), label, gutter);
    }
    out.push(bar(gutter));
    out
}

/// Byte range of `span` that falls on `line`, relative to that line's start.
///
/// # Arguments
///
/// * `map` — the source map.
/// * `span` — the span being rendered.
/// * `line` — 1-indexed line number.
///
/// # Returns
///
/// `(start, end)` byte offsets within the line's text. `end` may exceed the
/// line's length; [`caret_row`] clamps it.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, snippet::line_range};
/// let map = SourceMap::new("abc\ndef\n");
/// assert_eq!(line_range(&map, Span::new(1, 6), 1), (1, 6));
/// assert_eq!(line_range(&map, Span::new(1, 6), 2), (0, 2));
/// ```
pub fn line_range(map: &SourceMap, span: Span, line: usize) -> (usize, usize) {
    let base = map.line_start(line);
    (
        span.start.saturating_sub(base),
        span.end.saturating_sub(base),
    )
}

/// The caret row for `span` restricted to `line`.
///
/// # Arguments
///
/// * `map` — the source map.
/// * `span` — the span being rendered.
/// * `line` — 1-indexed line number.
/// * `continues` — whether the span extends past this line.
///
/// # Returns
///
/// The caret string, aligned in expanded-tab cell space.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, snippet::carets_for};
/// let map = SourceMap::new("let x = 1\n");
/// assert_eq!(carets_for(&map, Span::new(4, 5), 1, false), "    ^");
/// ```
pub fn carets_for(map: &SourceMap, span: Span, line: usize, continues: bool) -> String {
    let (a, b) = line_range(map, span, line);
    caret_row(map.line_text(line), a, b, continues)
}

pub use crate::diagnostic::snippet_rows::push_marks;
