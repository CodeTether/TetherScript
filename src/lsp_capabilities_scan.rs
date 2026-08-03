//! Token stream with byte offsets for one document.
//!
//! The scanner reuses the real [`Lexer`], so comments, string literals, and
//! interpolation holes are handled exactly the way the compiler handles them —
//! no regex approximation of TetherScript syntax, which is what the VSCode
//! client had to resort to in JavaScript.
//!
//! [`Spanned`] carries a 1-based line and a 1-based column, and the lexer
//! advances `col` once per **byte**. So a token's byte offset is
//! `line_start(line - 1) + (col - 1)`. That relationship is why this module
//! converts to byte offsets once, up front, instead of letting each feature
//! re-derive them: byte offsets compose with `&str` slicing, whereas the LSP's
//! UTF-16 columns do not.
//!
//! Brace-matching helpers on [`Scanned`] live in
//! `src/lsp_capabilities_scan_braces.rs` to keep each file single-purpose.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::scan::scan;
//!
//! let scanned = scan("let x = 1\n").expect("lexes");
//! assert_eq!(scanned.offsets[0], 0);
//! assert_eq!(scanned.offsets[1], 4);
//! ```

use crate::lexer::Lexer;
use crate::token::Spanned;

/// A tokenized document paired with a byte offset per token.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// let scanned = scan("nil").expect("lexes");
/// assert_eq!(scanned.tokens.len(), scanned.offsets.len());
/// ```
pub struct Scanned {
    /// Tokens as produced by [`Lexer::tokenize`], including the trailing `Eof`.
    pub tokens: Vec<Spanned>,
    /// Byte offset of each token's first byte, parallel to `tokens`.
    pub offsets: Vec<usize>,
}

/// Tokenize `text` and compute a byte offset for every token.
///
/// # Arguments
///
/// * `text` — Full document text.
///
/// # Returns
///
/// `Some(Scanned)` on success, or `None` when the document does not lex. A
/// half-typed document is normal in an editor, so callers treat `None` as
/// "no symbols known" and still serve keyword and builtin completions rather
/// than failing the whole request.
///
/// # Errors
///
/// Infallible; lex failure is reported as `None`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// assert!(scan("fn main() { }").is_some());
/// assert!(scan("\"unterminated").is_none());
/// ```
pub fn scan(text: &str) -> Option<Scanned> {
    let tokens = Lexer::new(text).tokenize().ok()?;
    let starts = line_starts(text);
    let offsets = tokens
        .iter()
        .map(|token| {
            let base = starts.get(token.line.saturating_sub(1)).copied().unwrap_or(0);
            (base + token.col.saturating_sub(1)).min(text.len())
        })
        .collect();
    Some(Scanned { tokens, offsets })
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    starts.extend(
        text.bytes()
            .enumerate()
            .filter(|(_, byte)| *byte == b'\n')
            .map(|(index, _)| index + 1),
    );
    starts
}
