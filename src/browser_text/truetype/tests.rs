//! TrueType measurement regression tests.

use super::measure;

#[test]
fn variable_width_glyphs_have_different_advances() {
    let wide = measure("WWW", 16).0;
    let narrow = measure("iii", 16).0;
    assert!(wide > narrow);
}
