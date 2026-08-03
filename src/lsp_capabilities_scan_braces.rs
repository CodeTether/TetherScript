//! Brace matching over a scanned token stream.
//!
//! Scope ends are computed structurally from `{` / `}` tokens rather than from
//! the AST, because the AST does not retain source offsets. Working from tokens
//! also means an unclosed block — the normal state while typing — degrades to
//! "scope runs to end of file" instead of producing no symbols at all.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::scan::scan;
//!
//! let scanned = scan("fn f() { }").expect("lexes");
//! assert_eq!(scanned.body_end(0), 9);
//! ```

use crate::lsp_capabilities::scan::Scanned;
use crate::token::Token;

impl Scanned {
    /// Byte offset of the `}` that closes the block enclosing token `index`.
    ///
    /// # Arguments
    ///
    /// * `index` — Token index to start scanning forward from.
    ///
    /// # Returns
    ///
    /// The closing brace's byte offset, or [`usize::MAX`] when `index` sits at
    /// top level or inside an unclosed block, where a binding stays in scope to
    /// end of file.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::lsp_capabilities::scan::scan;
    /// let scanned = scan("fn f() { let a = 1 }").expect("lexes");
    /// assert_eq!(scanned.enclosing_end(0), usize::MAX);
    /// ```
    pub fn enclosing_end(&self, index: usize) -> usize {
        let mut depth = 0i32;
        for position in index..self.tokens.len() {
            match self.tokens[position].token {
                Token::LBrace => depth += 1,
                Token::RBrace if depth == 0 => return self.offsets[position],
                Token::RBrace => depth -= 1,
                _ => {}
            }
        }
        usize::MAX
    }

    /// Byte offset of the `}` matching the first `{` at or after `index`.
    ///
    /// Used to bound a function body: parameters and its locals stop being
    /// visible past the returned offset.
    ///
    /// # Arguments
    ///
    /// * `index` — Token index at or before the opening brace.
    ///
    /// # Returns
    ///
    /// The closing brace offset, or [`usize::MAX`] when there is no `{` left or
    /// the block is unclosed.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::lsp_capabilities::scan::scan;
    /// let scanned = scan("fn f() { }").expect("lexes");
    /// assert_eq!(scanned.body_end(0), 9);
    /// assert_eq!(scan("let x = 1").expect("lexes").body_end(0), usize::MAX);
    /// ```
    pub fn body_end(&self, index: usize) -> usize {
        let open = (index..self.tokens.len())
            .find(|position| matches!(self.tokens[*position].token, Token::LBrace));
        match open {
            Some(open) => self.enclosing_end(open + 1),
            None => usize::MAX,
        }
    }
}
