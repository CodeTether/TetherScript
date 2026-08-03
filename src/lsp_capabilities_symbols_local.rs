//! `let`, `for`, and `import` declarations.
//!
//! Each form binds exactly one name, so the three helpers here differ only in
//! which token follows the keyword and how the resulting scope is bounded:
//!
//! - `let [mut] name` — visible from its own offset to the end of the enclosing
//!   block, which is why it is *not* hoisted.
//! - `for name in iter { ... }` — visible only inside the loop body.
//! - `import "path" as alias` — a hoisted, file-scoped namespace binding; the
//!   raw path is kept in [`Symbol::detail`] so definition and hover can resolve
//!   the target file the way [`crate::modules`] does.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::symbol::SymbolKind;
//! use tetherscript::lsp_capabilities::symbols::collect;
//!
//! let found = collect("import \"./math.tether\" as math\nlet x = 1");
//! assert_eq!(found[0].kind, SymbolKind::Module);
//! assert_eq!(found[0].detail, "./math.tether");
//! assert_eq!(found[1].kind, SymbolKind::Local);
//! ```

use crate::lsp_capabilities::scan::Scanned;
use crate::lsp_capabilities::symbol::{Symbol, SymbolKind};
use crate::lsp_capabilities::symbols::ident_at;
use crate::token::Token;

/// Record a `let` or `let mut` binding.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Index of the `Let` token.
/// * `out` — Symbol list to append to.
///
/// # Returns
///
/// Nothing; appends at most one symbol.
///
/// # Errors
///
/// Infallible; `let` with no following identifier contributes nothing.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols_local::binding;
/// let scanned = scan("let mut total = 0").expect("lexes");
/// let mut out = Vec::new();
/// binding(&scanned, 0, &mut out);
/// assert_eq!(out[0].name, "total");
/// assert_eq!(out[0].signature, "let mut total");
/// ```
pub fn binding(scanned: &Scanned, index: usize, out: &mut Vec<Symbol>) {
    let mutable = matches!(
        scanned.tokens.get(index + 1).map(|token| &token.token),
        Some(Token::Mut)
    );
    let name_index = if mutable { index + 2 } else { index + 1 };
    if let Some((name, offset)) = ident_at(scanned, name_index) {
        let keyword = if mutable { "let mut" } else { "let" };
        let signature = format!("{keyword} {name}");
        let mut symbol = Symbol::new(name, SymbolKind::Local, &signature, offset);
        symbol.scope_end = scanned.enclosing_end(index);
        out.push(symbol);
    }
}

/// Record a `for name in ...` loop binding, scoped to the loop body.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Index of the `For` token.
/// * `out` — Symbol list to append to.
///
/// # Returns
///
/// Nothing; appends at most one symbol.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols_local::loop_binding;
/// let scanned = scan("for item in list { }").expect("lexes");
/// let mut out = Vec::new();
/// loop_binding(&scanned, 0, &mut out);
/// assert_eq!(out[0].signature, "for item");
/// ```
pub fn loop_binding(scanned: &Scanned, index: usize, out: &mut Vec<Symbol>) {
    if let Some((name, offset)) = ident_at(scanned, index + 1) {
        let signature = format!("for {name}");
        let mut symbol = Symbol::new(name, SymbolKind::Local, &signature, offset);
        symbol.scope_end = scanned.body_end(index);
        out.push(symbol);
    }
}

/// Record an `import "path" as alias` namespace binding.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Index of the `Import` token.
/// * `out` — Symbol list to append to.
///
/// # Returns
///
/// Nothing; appends at most one symbol whose `detail` is the import path.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols_local::import;
/// let scanned = scan("import \"./m.tether\" as m").expect("lexes");
/// let mut out = Vec::new();
/// import(&scanned, 0, &mut out);
/// assert_eq!(out[0].detail, "./m.tether");
/// ```
pub fn import(scanned: &Scanned, index: usize, out: &mut Vec<Symbol>) {
    let path = match scanned.tokens.get(index + 1).map(|token| &token.token) {
        Some(Token::Str(path)) => path.clone(),
        _ => return,
    };
    if let Some((name, offset)) = ident_at(scanned, index + 3) {
        let signature = format!("import \"{path}\" as {name}");
        let mut symbol = Symbol::new(name, SymbolKind::Module, &signature, offset);
        symbol.detail = path;
        symbol.hoisted = true;
        out.push(symbol);
    }
}
