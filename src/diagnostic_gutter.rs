//! Gutter row primitives shared by every snippet block.
//!
//! One responsibility: turn a piece of content into a rustc-shaped row whose
//! `|` separator sits at a fixed column derived from the gutter width. Keeping
//! this in one place is what makes the byte-exact rendering tests meaningful —
//! there is exactly one definition of the layout.
//!
//! Layout, for gutter width `g` (digits in the widest line number shown):
//!
//! ```text
//!  --> file:line:col     <- g spaces, then "--> "
//!   |                    <- g+1 spaces, then "|"
//! 2 | let x = 1          <- line number right-aligned in g, " | ", source
//!   |     ^ label        <- bar row, space, carets, space, label
//! ```

use crate::diagnostic::caret::expand_tabs;
use crate::diagnostic::pos::LineCol;

/// An empty gutter row: `"  |"` at gutter width 1.
///
/// # Arguments
///
/// * `gutter` — width of the line-number field.
///
/// # Returns
///
/// The bar row with no content.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::gutter::bar;
/// assert_eq!(bar(1), "  |");
/// assert_eq!(bar(3), "    |");
/// ```
pub fn bar(gutter: usize) -> String {
    format!("{}|", " ".repeat(gutter + 1))
}

/// The `--> file:line:col` locator row.
///
/// # Arguments
///
/// * `gutter` — width of the line-number field.
/// * `name` — file name to show.
/// * `at` — resolved position; its `char_col` is printed, matching terminal
///   convention (LSP's UTF-16 column is not a human-facing number).
///
/// # Returns
///
/// The locator row without a trailing newline.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{SourceMap, gutter::header};
/// let map = SourceMap::with_name("a.tether", "let x = 1\n");
/// assert_eq!(header(1, map.name(), map.locate(4)), " --> a.tether:1:5");
/// ```
pub fn header(gutter: usize, name: &str, at: LineCol) -> String {
    let pad = " ".repeat(gutter);
    format!("{pad}--> {name}:{}:{}", at.line, at.char_col)
}

/// A source row: right-aligned line number, bar, tab-expanded source text.
///
/// # Arguments
///
/// * `gutter` — width of the line-number field.
/// * `line_no` — 1-indexed line number to print.
/// * `text` — the source line, tabs unexpanded.
///
/// # Returns
///
/// The source row, with tabs expanded per [`expand_tabs`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::gutter::src_row;
/// assert_eq!(src_row(1, 2, "\tx"), "2 |     x");
/// ```
pub fn src_row(gutter: usize, line_no: usize, text: &str) -> String {
    format!("{:>w$} | {}", line_no, expand_tabs(text), w = gutter)
}

/// A caret row, optionally followed by an inline label.
///
/// # Arguments
///
/// * `gutter` — width of the line-number field.
/// * `carets` — caret string from [`crate::diagnostic::caretrow::caret_row`].
/// * `label` — inline message, or `None` for carets alone.
///
/// # Returns
///
/// The caret row without a trailing newline.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::gutter::mark_row;
/// assert_eq!(mark_row(1, "  ^^", Some("here")), "  |   ^^ here");
/// assert_eq!(mark_row(1, "^", None), "  | ^");
/// ```
pub fn mark_row(gutter: usize, carets: &str, label: Option<&str>) -> String {
    let mut row = format!("{} {}", bar(gutter), carets);
    if let Some(text) = label {
        row.push(' ');
        row.push_str(text);
    }
    row
}
