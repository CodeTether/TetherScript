//! Deciding which lines of a multi-line span get a caret row.
//!
//! Split from [`crate::diagnostic::snippet`] so "build the block" and "decide
//! the per-line marks" stay separate responsibilities, and so both files stay
//! inside the 50-line limit.
//!
//! Rule: the first and last lines of a span are marked; intermediate lines are
//! context only. The label rides on the last marked line, because that is where
//! the reader's eye finishes.

use crate::diagnostic::gutter::mark_row;
use crate::diagnostic::map::SourceMap;
use crate::diagnostic::snippet::carets_for;
use crate::diagnostic::span::Span;

/// Appends the caret row for `line`, if that line deserves one.
///
/// # Arguments
///
/// * `out` — row buffer to append to.
/// * `map` — source map for the file.
/// * `span` — the span being rendered.
/// * `lines` — `(current, first, last)` 1-indexed line numbers.
/// * `label` — inline message, attached only to the last marked line.
/// * `gutter` — line-number field width.
///
/// # Returns
///
/// Nothing; `out` grows by zero or one row.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, Span, snippet_rows::push_marks};
///
/// let map = SourceMap::new("abc\ndef\n");
/// let span = Span::new(1, 6);
///
/// let mut first: Vec<String> = Vec::new();
/// push_marks(&mut first, &map, span, (1, 1, 2), Some("x"), 1);
/// assert_eq!(first, vec!["  |  ^^ ...".to_string()]);
///
/// let mut last: Vec<String> = Vec::new();
/// push_marks(&mut last, &map, span, (2, 1, 2), Some("x"), 1);
/// assert_eq!(last, vec!["  | ^^ x".to_string()]);
/// ```
pub fn push_marks(
    out: &mut Vec<String>,
    map: &SourceMap,
    span: Span,
    lines: (usize, usize, usize),
    label: Option<&str>,
    gutter: usize,
) {
    let (line, first, last) = lines;
    if line != first && line != last {
        return;
    }
    let continues = line < last;
    let carets = carets_for(map, span, line, continues);
    let text = if continues { None } else { label };
    out.push(mark_row(gutter, &carets, text));
}
