//! Document symbol model shared by completion, hover, and definition.
//!
//! A [`Symbol`] is one name a TetherScript document introduces: a function, a
//! `let` binding, a function parameter, a `for` loop binding, or an imported
//! module namespace. Each records the byte offset of the *name* token, so
//! [`crate::lsp_capabilities::position::offset_position`] can turn it into an
//! LSP range, plus the byte offset at which its lexical scope ends.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::symbol::{Symbol, SymbolKind};
//!
//! let symbol = Symbol::new("total", SymbolKind::Local, "let total", 4);
//! assert_eq!(symbol.kind, SymbolKind::Local);
//! assert_eq!(symbol.offset, 4);
//! ```

/// What kind of name a [`Symbol`] introduces.
///
/// The variants drive both completion ranking (see
/// [`crate::lsp_capabilities::rank`]) and the LSP `CompletionItemKind` /
/// `SymbolKind` numbers reported to the editor.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbol::SymbolKind;
///
/// let kind = SymbolKind::Param;
/// match kind {
///     SymbolKind::Function => println!("fn"),
///     SymbolKind::Local => println!("let"),
///     SymbolKind::Param => println!("parameter"),
///     SymbolKind::Module => println!("import alias"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// `fn name(...)` declaration.
    Function,
    /// `let` / `let mut` binding, or a `for` loop binding.
    Local,
    /// A function parameter.
    Param,
    /// An `import "..." as alias` namespace binding.
    Module,
}

impl SymbolKind {
    /// LSP `CompletionItemKind` number for this symbol kind.
    ///
    /// # Returns
    ///
    /// `3` (Function), `6` (Variable), or `9` (Module), per the LSP spec.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::lsp_capabilities::symbol::SymbolKind;
    /// assert_eq!(SymbolKind::Function.completion_kind(), 3);
    /// assert_eq!(SymbolKind::Local.completion_kind(), 6);
    /// assert_eq!(SymbolKind::Module.completion_kind(), 9);
    /// ```
    pub fn completion_kind(self) -> i64 {
        match self {
            SymbolKind::Function => 3,
            SymbolKind::Local | SymbolKind::Param => 6,
            SymbolKind::Module => 9,
        }
    }
}

/// One name declared by a document, with its source offset and scope end.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::symbol::{Symbol, SymbolKind};
///
/// let mut symbol = Symbol::new("add", SymbolKind::Function, "add(a, b)", 3);
/// symbol.scope_end = 42;
/// assert!(symbol.visible_at(10));
/// assert!(!symbol.visible_at(99));
/// ```
#[derive(Debug, Clone)]
pub struct Symbol {
    /// The declared identifier.
    pub name: String,
    /// What kind of declaration introduced it.
    pub kind: SymbolKind,
    /// Human-readable signature shown in completion detail and hover.
    pub signature: String,
    /// Extra payload: the raw import path for [`SymbolKind::Module`], else empty.
    pub detail: String,
    /// Byte offset of the first byte of the name token.
    pub offset: usize,
    /// Byte offset at which the declaration's lexical scope ends.
    pub scope_end: usize,
    /// True when the declaration is hoisted and visible before its own offset.
    pub hoisted: bool,
}

impl Symbol {
    /// Create a symbol whose scope initially runs to the end of the document.
    ///
    /// # Arguments
    ///
    /// * `name` — Declared identifier.
    /// * `kind` — Declaration kind.
    /// * `signature` — Text shown to the user.
    /// * `offset` — Byte offset of the name token.
    ///
    /// # Returns
    ///
    /// A symbol with `scope_end` set to [`usize::MAX`] and `hoisted` false; the
    /// scanner narrows both as it closes blocks.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::lsp_capabilities::symbol::{Symbol, SymbolKind};
    /// let symbol = Symbol::new("x", SymbolKind::Local, "let x", 4);
    /// assert_eq!(symbol.scope_end, usize::MAX);
    /// ```
    pub fn new(name: &str, kind: SymbolKind, signature: &str, offset: usize) -> Self {
        Self {
            name: name.to_string(),
            kind,
            signature: signature.to_string(),
            detail: String::new(),
            offset,
            scope_end: usize::MAX,
            hoisted: false,
        }
    }

    /// Whether this symbol is in scope at a cursor byte `offset`.
    ///
    /// Non-hoisted bindings become visible only after their own declaration;
    /// hoisted ones (top-level `fn`s and imports) are visible anywhere in their
    /// block, matching TetherScript's top-level hoisting.
    ///
    /// # Arguments
    ///
    /// * `offset` — Cursor byte offset.
    ///
    /// # Returns
    ///
    /// `true` when the name resolves at that cursor position.
    ///
    /// # Errors
    ///
    /// Infallible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::lsp_capabilities::symbol::{Symbol, SymbolKind};
    /// let mut symbol = Symbol::new("x", SymbolKind::Local, "let x", 10);
    /// symbol.scope_end = 20;
    /// assert!(!symbol.visible_at(5));
    /// assert!(symbol.visible_at(15));
    /// ```
    pub fn visible_at(&self, offset: usize) -> bool {
        offset <= self.scope_end && (self.hoisted || offset >= self.offset)
    }
}
