//! `fn` declarations and their parameters.
//!
//! Handles both `fn name(a, b) { ... }` and `async fn name(a, b) { ... }`: the
//! lexer emits `Async` before `Fn`, so this module only has to key off `Fn`.
//! Anonymous `fn(...)` expressions declare no name, so only their parameters
//! are recorded.
//!
//! Top-level functions are marked hoisted, matching the language's top-level
//! hoisting (see [`crate::ast::Stmt::FnDecl`]); a call above the declaration
//! must still resolve, or go-to-definition would fail on the most common layout
//! in the repository's own examples, where `main` sits at the bottom.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::symbols::collect;
//! use tetherscript::lsp_capabilities::symbol::SymbolKind;
//!
//! let found = collect("fn add(a, b) { a }");
//! assert_eq!(found[0].signature, "add(a, b)");
//! assert_eq!(found[1].kind, SymbolKind::Param);
//! ```

use crate::lsp_capabilities::scan::Scanned;
use crate::lsp_capabilities::symbol::{Symbol, SymbolKind};
use crate::lsp_capabilities::symbols::ident_at;
use crate::token::Token;

/// Record the function named at `index` (if any) plus all of its parameters.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Index of the `Fn` token.
/// * `depth` — Brace depth before the `Fn` token; `0` means top level.
/// * `out` — Symbol list to append to.
///
/// # Returns
///
/// Nothing; results are appended to `out`.
///
/// # Errors
///
/// Infallible; malformed headers simply contribute fewer symbols.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols_fn::declaration;
/// let scanned = scan("fn f(x) { }").expect("lexes");
/// let mut out = Vec::new();
/// declaration(&scanned, 0, 0, &mut out);
/// assert_eq!(out.len(), 2);
/// ```
pub fn declaration(scanned: &Scanned, index: usize, depth: i32, out: &mut Vec<Symbol>) {
    let spans = param_spans(scanned, index);
    let body_end = scanned.body_end(index);
    if let Some((name, offset)) = ident_at(scanned, index + 1) {
        let names: Vec<&str> = spans.iter().map(|(name, _)| name.as_str()).collect();
        let signature = format!("{name}({})", names.join(", "));
        let mut symbol = Symbol::new(name, SymbolKind::Function, &signature, offset);
        symbol.hoisted = true;
        symbol.scope_end = if depth == 0 {
            usize::MAX
        } else {
            scanned.enclosing_end(index)
        };
        out.push(symbol);
    }
    for (name, offset) in spans {
        let mut symbol = Symbol::new(&name, SymbolKind::Param, &name, offset);
        symbol.scope_end = body_end;
        out.push(symbol);
    }
}

fn param_spans(scanned: &Scanned, index: usize) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let open = match (index..scanned.tokens.len())
        .find(|position| matches!(scanned.tokens[*position].token, Token::LParen))
    {
        Some(open) => open,
        None => return out,
    };
    for position in (open + 1)..scanned.tokens.len() {
        match scanned.tokens[position].token {
            Token::RParen | Token::LBrace | Token::Eof => break,
            Token::Ident(ref name) => out.push((name.clone(), scanned.offsets[position])),
            _ => {}
        }
    }
    out
}

/// Parameter names declared by the function header starting at `index`.
///
/// # Arguments
///
/// * `scanned` — Scanned document.
/// * `index` — Index of the `Fn` token.
///
/// # Returns
///
/// Parameter names in declaration order; empty for `fn f()`.
///
/// # Errors
///
/// Infallible.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::scan::scan;
/// use tetherscript::lsp_capabilities::symbols_fn::params;
/// let scanned = scan("fn f(a, b) { }").expect("lexes");
/// assert_eq!(params(&scanned, 0), vec!["a".to_string(), "b".to_string()]);
/// ```
pub fn params(scanned: &Scanned, index: usize) -> Vec<String> {
    param_spans(scanned, index)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}
