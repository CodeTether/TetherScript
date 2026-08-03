//! # Formatting the reassembled `numeric` as a decimal string
//!
//! Splits into an integer part and a fraction part because the two have different
//! padding rules, and both differ from the naive "concatenate the groups" approach
//! that produces wrong answers at the edges:
//!
//! - **Integer part.** Exponents `weight` down to `0`, each group rendered as four
//!   zero-padded digits, then leading zeros stripped. Uniform padding is what keeps
//!   an interior group like `7` as `0007` instead of merging into the digit above
//!   it; stripping afterwards is what keeps the leading `1` of `12345` from being
//!   printed as `0001`. A `weight` below zero means there is no integer part at all
//!   and the result starts `0`.
//! - **Fraction part.** Exactly `dscale` digits, taken from exponents `-1`
//!   downward, four digits per group, always zero-padded, then truncated to
//!   `dscale`. Missing groups are implicit zeros, so a value with a large negative
//!   weight still gets its leading fractional zeros.
//!
//! `dscale == 0` emits no decimal point at all: `5`, not `5.`.
//!
//! No floating point is used. The sign is taken from the header verbatim rather
//! than inferred, so the rendering is faithful to what the server sent.

use super::digits::{group_at, render_group_run};
use super::header::Header;

/// Render a finite `numeric` from its header and digit groups.
///
/// # Arguments
///
/// * `header` — validated header supplying `weight`, `sign`, and `dscale`.
/// * `groups` — the transmitted base-10000 digit groups.
///
/// # Returns
///
/// The exact decimal string, e.g. `12345.6789`, `-0.50`, or `0`.
pub(super) fn render(header: &Header, groups: &[u16]) -> String {
    let mut out = String::new();
    if header.negative() {
        out.push('-');
    }
    out.push_str(&integer_part(header, groups));
    if header.dscale > 0 {
        out.push('.');
        out.push_str(&fraction_part(header, groups));
    }
    out
}

/// Digits at exponents `weight..=0`, or `"0"` when `weight` is negative.
fn integer_part(header: &Header, groups: &[u16]) -> String {
    if header.weight < 0 {
        return "0".into();
    }
    let exponents = (0..=header.weight as i32).rev();
    let digits = render_group_run(header, groups, exponents);
    // Uniform 4-digit padding means an all-zero integer part is all zeros here,
    // and a leading group of 1 arrives as "0001"; strip to the canonical form.
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        return "0".into();
    }
    trimmed.to_string()
}

/// Exactly `dscale` digits at exponents `-1, -2, …`, zero-padded then truncated.
fn fraction_part(header: &Header, groups: &[u16]) -> String {
    // Four decimal digits per base-10000 group; round the group count up.
    let needed_groups = header.dscale.div_ceil(4);
    let mut digits = String::with_capacity(needed_groups * 4);
    for step in 1..=needed_groups as i32 {
        let group = group_at(header, groups, -step);
        digits.push_str(&format!("{group:04}"));
    }
    digits.truncate(header.dscale);
    digits
}
