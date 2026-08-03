//! Completion ranking.
//!
//! The LSP has no numeric score field, so ranking is expressed through
//! `sortText`: clients sort completion items by `sortText` (falling back to
//! `label`), so a stable zero-padded prefix is how a server states its own
//! ordering instead of letting the client sort alphabetically.
//!
//! ## Tiers, most specific first
//!
//! | Tier | Contents | Why here |
//! |------|----------|----------|
//! | 0 | Exact-prefix in-scope locals and parameters | The user's own nearby names are what they are most likely typing. |
//! | 1 | Other in-scope locals and parameters | Still theirs, but not prefix-matching. |
//! | 2 | Functions declared in this file | File-local, but wider in scope than a local. |
//! | 3 | Imported module aliases | One indirection away. |
//! | 4 | Builtins | Global, numerous; would otherwise drown out local names. |
//! | 5 | Keywords | Short and already memorised; users type them faster than they pick them. |
//! | 6 | Constants (`true`, `false`, `nil`) | Shortest of all. |
//!
//! Within a tier, items are ordered by the natural insertion order of their
//! source table, then alphabetically by the client's `label` fallback. Locals
//! additionally sort by *proximity*: the declaration closest above the cursor
//! wins, because in a long function the nearest `let` is nearly always the
//! intended one.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::rank::{Tier, sort_text};
//!
//! assert!(sort_text(Tier::LocalExact, 0) < sort_text(Tier::Builtin, 0));
//! assert!(sort_text(Tier::Local, 1) < sort_text(Tier::Local, 9));
//! ```

/// A ranking tier, ordered most-specific first.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::rank::Tier;
/// assert!((Tier::LocalExact as u8) < (Tier::Keyword as u8));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// In-scope local or parameter whose name starts with the typed prefix.
    LocalExact,
    /// Any other in-scope local or parameter.
    Local,
    /// A function declared in the current file.
    Function,
    /// An imported module namespace alias.
    Module,
    /// A built-in function.
    Builtin,
    /// A language keyword.
    Keyword,
    /// A literal constant.
    Constant,
}

/// Build a client-sortable `sortText` for a tier and within-tier position.
///
/// # Arguments
///
/// * `tier` — Ranking tier.
/// * `within` — Position inside the tier; smaller sorts earlier. For locals this
///   is the number of intervening declarations between the cursor and the
///   binding, so nearer bindings sort first.
///
/// # Returns
///
/// A string of the form `"<tier><within padded to 4 digits>"`, e.g. `"00012"`.
/// Zero-padding is what makes lexicographic client sorting agree with numeric
/// intent; `"12"` would otherwise sort before `"2"`.
///
/// # Errors
///
/// Infallible; `within` is clamped to 9999 so the width never changes.
///
/// # Examples
///
/// ```rust
/// use tetherscript::lsp_capabilities::rank::{Tier, sort_text};
/// assert_eq!(sort_text(Tier::LocalExact, 12), "00012");
/// assert_eq!(sort_text(Tier::Constant, 0), "60000");
/// assert_eq!(sort_text(Tier::Local, 100000), "19999");
/// ```
pub fn sort_text(tier: Tier, within: usize) -> String {
    format!("{}{:04}", tier as u8, within.min(9999))
}
