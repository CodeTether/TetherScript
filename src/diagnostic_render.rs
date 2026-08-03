//! Full diagnostic reports: severity, message, a primary span, and related spans.
//!
//! ## Why related spans exist
//!
//! Ownership errors are the motivating case. "use of moved value `x`" at one
//! location is not actionable: the reader needs to see *where* it moved. A
//! report therefore carries one primary [`Label`] plus any number of related
//! labels, each rendered as its own snippet block with its own `-->` locator, so
//! a move and a later use are both on screen even when they are hundreds of
//! lines apart.
//!
//! ## Rendered shape
//!
//! ```text
//! error: use of moved value `x`
//!  --> a.tether:2:3
//!   |
//! 2 | f(x);
//!   |   ^ value used here after move
//!   |
//! note: value moved here
//!  --> a.tether:1:9
//!   |
//! 1 | let y = move x;
//!   |         ^^^^^^
//!   |
//! ```
//!
//! Construction lives in [`crate::diagnostic::build`] and
//! [`crate::diagnostic::relate`]; this file owns only the printed form.

use crate::diagnostic::label::Label;
use crate::diagnostic::map::SourceMap;
use crate::diagnostic::severity::Severity;
use crate::diagnostic::snippet::block;

/// A complete diagnostic: severity, message, primary label, related labels.
///
/// Build with [`Diagnostic::error`] / [`Diagnostic::warning`] plus
/// [`Diagnostic::with_related`]; print with [`Diagnostic::render`].
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{Diagnostic, Severity, Span};
///
/// let d = Diagnostic::error(Span::new(0, 3), "unexpected character `#`");
/// assert_eq!(d.severity, Severity::Error);
/// assert_eq!(d.primary.span, Span::new(0, 3));
/// assert!(d.related.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// How serious the problem is.
    pub severity: Severity,
    /// The headline message, printed after `error: ` / `warning: `.
    pub message: String,
    /// The span primarily at fault.
    pub primary: Label,
    /// Supporting locations, e.g. where a value was moved.
    pub related: Vec<Label>,
}

/// Width of the line-number gutter for a report.
///
/// # Arguments
///
/// * `map` — the source map being rendered against.
/// * `diag` — the diagnostic, whose largest referenced line sets the width.
///
/// # Returns
///
/// The number of decimal digits needed, at least 1, so every block in one
/// report aligns on the same `|` column.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::{Diagnostic, SourceMap, Span, render::gutter_width};
///
/// let map = SourceMap::new(&"x\n".repeat(120));
/// assert_eq!(gutter_width(&map, &Diagnostic::error(Span::new(0, 1), "m")), 1);
/// assert_eq!(gutter_width(&map, &Diagnostic::error(Span::new(200, 201), "m")), 3);
/// ```
pub fn gutter_width(map: &SourceMap, diag: &Diagnostic) -> usize {
    let mut max = map.locate(diag.primary.span.end).line;
    for rel in &diag.related {
        max = max.max(map.locate(rel.span.end).line);
    }
    max.to_string().len()
}

impl Diagnostic {
    /// Renders the whole report, rustc style.
    ///
    /// # Arguments
    ///
    /// * `map` — source map for the file every span in this report refers to.
    ///
    /// # Returns
    ///
    /// A newline-terminated multi-line string. Each related label is preceded by
    /// its own `note: <message>` header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::{Diagnostic, SourceMap, Span};
    ///
    /// let map = SourceMap::with_name("a.tether", "let x = 1\n");
    /// let out = Diagnostic::error(Span::new(4, 5), "oops")
    ///     .with_primary_label("this one")
    ///     .render(&map);
    /// assert_eq!(
    ///     out,
    ///     concat!(
    ///         "error: oops\n",
    ///         " --> a.tether:1:5\n",
    ///         "  |\n",
    ///         "1 | let x = 1\n",
    ///         "  |     ^ this one\n",
    ///         "  |\n",
    ///     )
    /// );
    /// ```
    pub fn render(&self, map: &SourceMap) -> String {
        let g = gutter_width(map, self);
        let mut rows = vec![format!("{}: {}", self.severity.as_str(), self.message)];
        rows.extend(block(map, self.primary.span, self.primary.text(), g));
        for rel in &self.related {
            rows.push(format!("note: {}", rel.message));
            rows.extend(block(map, rel.span, None, g));
        }
        rows.join("\n") + "\n"
    }
}
