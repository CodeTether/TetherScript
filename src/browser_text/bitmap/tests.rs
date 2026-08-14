//! Deterministic bitmap measurement regression tests.

use super::measure;

#[test]
fn every_glyph_uses_one_fixed_width_cell() {
    assert_eq!(measure("WWW", 16), (48, 16));
    assert_eq!(measure("iii", 16), (48, 16));
}

#[test]
fn measurement_clamps_non_positive_sizes() {
    assert_eq!(measure("text", 0), (4, 1));
}
