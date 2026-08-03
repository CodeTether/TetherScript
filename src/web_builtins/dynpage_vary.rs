//! `Vary` header derivation.
//!
//! # Exactly the headers the key consumed
//!
//! `Vary` tells a shared cache which request headers it must compare before
//! reusing a stored response. It must therefore list precisely the headers that
//! fed [`super::dynpage_key`] — no more, no less — and the trade-off is sharp in
//! both directions.
//!
//! * **Too few.** The cache stops distinguishing an input the render actually
//!   depends on. Omit `Accept-Language` while the key uses a locale and an
//!   intermediary happily serves the Spanish copy to an English reader; that is
//!   cache poisoning reachable by any visitor who arrives first.
//! * **Too many.** Every listed header multiplies the number of stored variants.
//!   Listing `User-Agent` unconditionally is the classic mistake: the header is
//!   effectively unique per client, so the hit rate collapses to roughly zero and
//!   the cache becomes pure overhead while appearing to work.
//!
//! # Mapping
//!
//! | Key component | Header listed |
//! |---|---|
//! | `locale` | `Accept-Language` |
//! | `device` | `User-Agent` |
//! | `authenticated` | `Cookie` and `Authorization` |
//! | `slug`, `variant` | *none* |
//!
//! `slug` comes from the path, not a header. `variant` comes from an assignment
//! the caller already resolved — a cookie in `abtest`'s sticky case — and is
//! covered by the `Cookie` entry only when the render is authenticated; a caller
//! whose variant lives in a cookie for an anonymous visitor must add `Cookie`
//! itself, because this group cannot see where the variant came from.
//!
//! `Vary: User-Agent` appears **only** when the key actually consumed a device
//! class, which is why `device_class` is an opt-in input rather than something
//! folded in unconditionally.

use super::dynpage_parts::Parts;

/// Build the `Vary` header value for a set of render inputs.
///
/// # Arguments
///
/// * `parts` — The same inputs passed to [`super::dynpage_key::build`].
///
/// # Returns
///
/// A comma-separated header value in a fixed order, or the empty string when the
/// key consumed no request headers at all. An empty string means the caller should
/// omit the header rather than send `Vary:` with no value.
pub(super) fn build(parts: &Parts) -> String {
    let mut names: Vec<&str> = Vec::new();
    if !parts.locale.is_empty() {
        names.push("Accept-Language");
    }
    if !parts.device.is_empty() {
        names.push("User-Agent");
    }
    if parts.authenticated {
        // Both, because either can carry the credential that made it private.
        names.push("Cookie");
        names.push("Authorization");
    }
    names.join(", ")
}
