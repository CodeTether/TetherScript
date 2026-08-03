//! Caret rows and tab handling for terminal rendering.
//!
//! ## Tab decision (documented behaviour, not an accident)
//!
//! A caret row only lines up under the source line if both are measured in the
//! same units. A `char` count is *not* that unit when the line contains tabs: a
//! terminal advances a tab to the next tab stop, so a single `\t` can occupy one
//! to [`TAB_WIDTH`] cells.
//!
//! We therefore **expand tabs to spaces at `TAB_WIDTH = 4` stops in the
//! rendered source line** and compute caret offsets in that same expanded cell
//! space. The rendered line is not byte-identical to the file, but the carets
//! are guaranteed to sit under the right characters in every terminal,
//! regardless of the reader's tab setting. Columns reported in
//! [`crate::diagnostic::LineCol`] are untouched by this: they stay honest
//! character/UTF-16 counts.
//!
//! Non-tab characters are counted as one cell each. We deliberately do not
//! implement East Asian Wide or zero-width/combining-mark handling: it needs
//! Unicode tables we will not take a dependency for, and the failure mode is a
//! cosmetic one-cell drift rather than a wrong span.

/// Cells a tab advances to (tab stops every 4 columns).
pub const TAB_WIDTH: usize = 4;

/// Cells consumed by `text` when rendered starting at cell `start_cell`.
///
/// # Arguments
///
/// * `text` — the text to measure.
/// * `start_cell` — 0-based cell the text starts at, needed because a tab's
///   width depends on where it begins.
///
/// # Returns
///
/// The number of terminal cells `text` occupies.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::caret;
///
/// assert_eq!(caret::display_width_at("ab", 0), 2);
/// assert_eq!(caret::display_width_at("\t", 0), 4); // 0 -> next stop 4
/// assert_eq!(caret::display_width_at("\t", 3), 1); // 3 -> next stop 4
/// ```
pub fn display_width_at(text: &str, start_cell: usize) -> usize {
    let mut cells = start_cell;
    for ch in text.chars() {
        cells += if ch == '\t' {
            TAB_WIDTH - (cells % TAB_WIDTH)
        } else {
            1
        };
    }
    cells - start_cell
}

/// Cells consumed by `text` when rendered from the start of a line.
///
/// # Arguments
///
/// * `text` — the text to measure.
///
/// # Returns
///
/// [`display_width_at`] with `start_cell = 0`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::caret;
/// assert_eq!(caret::display_width("\tx"), 5);
/// ```
pub fn display_width(text: &str) -> usize {
    display_width_at(text, 0)
}

/// Expands tabs in `line` to spaces at [`TAB_WIDTH`] stops.
///
/// # Arguments
///
/// * `line` — one source line, without its terminator.
///
/// # Returns
///
/// The line with every tab replaced by enough spaces to reach the next tab
/// stop, so carets computed in cell space align under it.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::caret;
///
/// assert_eq!(caret::expand_tabs("\tlet x"), "    let x");
/// assert_eq!(caret::expand_tabs("ab\tc"), "ab  c");
/// ```
pub fn expand_tabs(line: &str) -> String {
    let mut out = String::new();
    let mut cells = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            let pad = TAB_WIDTH - (cells % TAB_WIDTH);
            out.extend(std::iter::repeat_n(' ', pad));
            cells += pad;
        } else {
            out.push(ch);
            cells += 1;
        }
    }
    out
}
