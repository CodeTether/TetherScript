//! # Exact diagnostic spans
//!
//! Byte-offset spans, a fast offset → line/column source map, rustc-style
//! terminal rendering, and LSP (UTF-16) range conversion for tetherscript
//! diagnostics.
//!
//! ## Why byte offsets are the primitive
//!
//! A diagnostic position is fundamentally *a region of the source buffer*, and
//! the only lossless way to name a region of a UTF-8 buffer is a pair of byte
//! offsets. Line/column is a **presentation** concern: it is derived, and it is
//! ambiguous (is a column a byte, a `char`, a grapheme, a UTF-16 code unit, or a
//! terminal cell?), with different consumers needing different answers. So the
//! lexer and parser record [`Span`] and let [`SourceMap`] answer every
//! presentation question. That ordering also makes spans cheap to merge
//! ([`Span::join`]) and cheap to compare, which line/column pairs are not — you
//! cannot union two line/column pairs without the text.
//!
//! ## Quick start
//!
//! ```rust
//! use tetherscript::diagnostic::{Diagnostic, SourceMap, Span};
//!
//! let src = "let y = move x;\nf(x);\n";
//! let map = SourceMap::with_name("a.tether", src);
//! let report = Diagnostic::error(Span::new(18, 19), "use of moved value `x`")
//!     .with_primary_label("value used here after move")
//!     .with_related(Span::new(8, 14), "value moved here");
//!
//! assert_eq!(
//!     report.render(&map),
//!     concat!(
//!         "error: use of moved value `x`\n",
//!         " --> a.tether:2:3\n",
//!         "  |\n",
//!         "2 | f(x);\n",
//!         "  |   ^ value used here after move\n",
//!         "  |\n",
//!         "note: value moved here\n",
//!         " --> a.tether:1:9\n",
//!         "  |\n",
//!         "1 | let y = move x;\n",
//!         "  |         ^^^^^^\n",
//!         "  |\n",
//!     )
//! );
//! ```
//!
//! ## Module map
//!
//! * [`span`] — the [`Span`] type and its arithmetic.
//! * [`pos`] — [`LineCol`], the three column flavours.
//! * [`utf16`] — UTF-16 code-unit counting and its inverse.
//! * [`map`] — [`SourceMap`] construction and line access.
//! * [`locate`] — offset → [`LineCol`] in `O(log L)`.
//! * [`offset`] — line/column → offset, the inverse direction.
//! * [`caret`] — tab expansion and display widths.
//! * [`caretrow`] — the caret run itself.
//! * [`gutter`] — the row layout primitives.
//! * [`snippet`] / [`snippet_rows`] — one source block.
//! * [`render`] / [`build`] / [`relate`] — [`Diagnostic`] assembly and printing.
//! * [`lsp`] — [`LspRange`] conversion with UTF-16 characters.

#[path = "diagnostic_build.rs"]
pub mod build;
#[path = "diagnostic_caret.rs"]
pub mod caret;
#[path = "diagnostic_caretrow.rs"]
pub mod caretrow;
#[path = "diagnostic_gutter.rs"]
pub mod gutter;
#[path = "diagnostic_label.rs"]
pub mod label;
#[path = "diagnostic_locate.rs"]
pub mod locate;
#[path = "diagnostic_lsp.rs"]
pub mod lsp;
#[path = "diagnostic_map.rs"]
pub mod map;
#[path = "diagnostic_offset.rs"]
pub mod offset;
#[path = "diagnostic_pos.rs"]
pub mod pos;
#[path = "diagnostic_relate.rs"]
pub mod relate;
#[path = "diagnostic_render.rs"]
pub mod render;
#[path = "diagnostic_severity.rs"]
pub mod severity;
#[path = "diagnostic_snippet.rs"]
pub mod snippet;
#[path = "diagnostic_snippet_rows.rs"]
pub mod snippet_rows;
#[path = "diagnostic_span.rs"]
pub mod span;
#[path = "diagnostic_utf16.rs"]
pub mod utf16;

pub use caret::TAB_WIDTH;
pub use label::Label;
pub use lsp::{lsp_range, LspPosition, LspRange};
pub use map::SourceMap;
pub use pos::LineCol;
pub use render::Diagnostic;
pub use severity::Severity;
pub use span::Span;
