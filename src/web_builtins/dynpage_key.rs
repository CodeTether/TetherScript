//! Cache-key derivation.
//!
//! # Every varying input, or the key is wrong
//!
//! A cache key names the *set of inputs* a stored render was produced from. Omit
//! one and two different renders land on one key, so whichever was stored first is
//! served to everybody:
//!
//! * **Omit `authenticated`** and a page rendered for a signed-in visitor — their
//!   name, their basket, their account number — is stored under the same key an
//!   anonymous visitor computes, and the next anonymous request is served that
//!   visitor's private page. That is a cross-user data leak, not a stale-content
//!   annoyance.
//! * **Omit `locale`** and the first visitor to warm the entry picks the language
//!   for everyone: a Spanish reader stores `es`, and every later English reader is
//!   served Spanish.
//! * **Omit `variant`** and an A/B experiment reports noise, because subjects see
//!   whichever arm happened to be cached rather than the arm they were assigned.
//! * **Omit `device`** and a phone is served the desktop layout, or the desktop is
//!   served the phone layout with its stripped-down content.
//!
//! # Separator
//!
//! Components are joined with a single **ASCII Unit Separator, `0x1F`**. It cannot
//! appear inside a component, because every component is either a fixed literal or
//! has passed the `[a-z0-9_-]` allowlist in [`super::dynpage_charset`], which
//! admits no control byte at all. That is what makes the join injective:
//! `a` + `bc` gives `a<US>bc` while `ab` + `c` gives `ab<US>c`, two different
//! strings, so no two distinct component tuples can produce one key. A printable
//! separator such as `-` or `:` would **not** have this property, since `-` is
//! legal inside a slug and `ab-c` would then be ambiguous.
//!
//! # Private renders
//!
//! An authenticated key is emitted with the literal first component `private`
//! instead of `public`, so the key reads `private<US>v1<US>…`. A shared cache
//! excludes it with one prefix test and never has to understand the rest of the
//! key. The marker is deliberately *first* for exactly that reason, and the key
//! still carries the remaining inputs so a per-session private cache stays usable.

use super::dynpage_parts::Parts;

/// Unambiguous component separator. See the module docs for why `0x1F`.
const SEP: &str = "\u{1f}";

/// Key scheme version, so a change to the layout invalidates old entries rather
/// than silently reinterpreting them.
const VERSION: &str = "v1";

/// Build the cache key for a set of render inputs.
///
/// # Arguments
///
/// * `parts` — The validated render inputs.
///
/// # Returns
///
/// The key: visibility marker, scheme version, slug, locale, variant, and device
/// class, joined by `0x1F`. An absent optional input appears as an empty
/// component, which keeps the component count fixed and the key unambiguous.
pub(super) fn build(parts: &Parts) -> String {
    let visibility = if parts.authenticated {
        "private"
    } else {
        "public"
    };
    [
        visibility,
        VERSION,
        parts.slug.as_str(),
        parts.locale.as_str(),
        parts.variant.as_str(),
        parts.device.as_str(),
    ]
    .join(SEP)
}
