//! Terminal display width measurement.
//!
//! Terminal cells are not Unicode scalar values: CJK ideographs and most
//! emoji occupy two columns, while combining marks and variation selectors
//! occupy none. Padding a line by [`str::chars`] count therefore overflows
//! or under-fills the frame. This module measures true column width.

use super::wide_astral::WIDE_ASTRAL;
use super::wide_bmp::WIDE_BMP;
use super::zero_high::ZERO_HIGH;
use super::zero_low::ZERO_LOW;

/// Return the number of terminal columns `ch` occupies.
///
/// Combining marks and zero-width formatting characters yield `0`, East
/// Asian Wide and Fullwidth characters yield `2`, everything else `1`.
pub(crate) fn char_width(ch: char) -> usize {
    let cp = ch as u32;
    if cp == 0 || in_table(cp, ZERO_LOW) || in_table(cp, ZERO_HIGH) {
        return 0;
    }
    if in_table(cp, WIDE_BMP) || in_table(cp, WIDE_ASTRAL) {
        return 2;
    }
    1
}

/// Return the number of terminal columns `text` occupies.
///
/// ANSI escape sequences are not skipped; callers that may pass styled
/// text should strip or step over escapes first.
#[cfg(test)]
pub(crate) fn str_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

/// Binary search a sorted, non-overlapping range table for `cp`.
fn in_table(cp: u32, table: &[(u32, u32)]) -> bool {
    table
        .binary_search_by(|&(lo, hi)| {
            if cp < lo {
                std::cmp::Ordering::Greater
            } else if cp > hi {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}
