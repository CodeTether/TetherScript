//! TrueType text measurement shared with layout.

use fontdue::Font;

pub(super) fn text(font: &Font, text: &str, size: f32) -> (i64, i64) {
    let mut width = 0.0_f32;
    let mut max_width = 0.0_f32;
    let mut lines = 1_i64;
    for character in text.chars() {
        if character == '\n' {
            max_width = max_width.max(width);
            width = 0.0;
            lines += 1;
        } else {
            width += font.metrics(character, size).advance_width;
        }
    }
    (
        max_width.max(width).ceil() as i64,
        (size * 1.25).ceil() as i64 * lines,
    )
}
