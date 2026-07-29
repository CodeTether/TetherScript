//! Partition invariants across the generated width range tables.
//!
//! The tables are split by plane and by width class. Zero-width must never
//! intersect wide, or `measure::char_width` would depend on probe order.

use super::wide_astral::WIDE_ASTRAL;
use super::wide_bmp::WIDE_BMP;
use super::zero_high::ZERO_HIGH;
use super::zero_low::ZERO_LOW;

#[test]
fn wide_tables_are_plane_partitioned() {
    for &(lo, hi) in WIDE_BMP {
        assert!(hi < 0x10000, "WIDE_BMP range 0x{lo:X} escapes the BMP");
    }
    for &(lo, _) in WIDE_ASTRAL {
        assert!(
            lo >= 0x10000,
            "WIDE_ASTRAL range 0x{lo:X} is inside the BMP"
        );
    }
}

#[test]
fn zero_tables_are_disjoint_and_ordered() {
    let low_end = ZERO_LOW.last().expect("ZERO_LOW is non-empty").1;
    let high_start = ZERO_HIGH.first().expect("ZERO_HIGH is non-empty").0;
    assert!(
        low_end < high_start,
        "ZERO_LOW ends at 0x{low_end:X} but ZERO_HIGH starts at 0x{high_start:X}"
    );
}

#[test]
fn zero_and_wide_tables_do_not_intersect() {
    for &(lo, hi) in ZERO_LOW.iter().chain(ZERO_HIGH) {
        for &(wlo, whi) in WIDE_BMP.iter().chain(WIDE_ASTRAL) {
            assert!(
                hi < wlo || whi < lo,
                "zero range (0x{lo:X}, 0x{hi:X}) overlaps wide (0x{wlo:X}, 0x{whi:X})"
            );
        }
    }
}
