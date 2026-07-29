//! Sort and overlap invariants for the generated width range tables.
//!
//! Binary search in `measure::in_table` is only correct when every table
//! is sorted, non-inverted, and free of overlapping ranges.

use super::wide_astral::WIDE_ASTRAL;
use super::wide_bmp::WIDE_BMP;
use super::zero_high::ZERO_HIGH;
use super::zero_low::ZERO_LOW;

fn assert_sorted(table: &[(u32, u32)], name: &str) {
    for &(lo, hi) in table {
        assert!(lo <= hi, "{name}: range (0x{lo:X}, 0x{hi:X}) is inverted");
    }
    for pair in table.windows(2) {
        let (_, prev_hi) = pair[0];
        let (next_lo, _) = pair[1];
        assert!(
            prev_hi < next_lo,
            "{name}: 0x{prev_hi:X} and 0x{next_lo:X} overlap or are unsorted"
        );
    }
}

#[test]
fn all_tables_are_sorted_and_non_overlapping() {
    assert_sorted(WIDE_BMP, "WIDE_BMP");
    assert_sorted(WIDE_ASTRAL, "WIDE_ASTRAL");
    assert_sorted(ZERO_LOW, "ZERO_LOW");
    assert_sorted(ZERO_HIGH, "ZERO_HIGH");
}
