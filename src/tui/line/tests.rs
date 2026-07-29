//! Tests for column-exact line fitting in terminal frames.

use super::fit as line;
use super::tests_width::visible_width;

#[test]
fn pads_ascii_to_exact_columns() {
    assert_eq!(visible_width(&line::fit("ab", 5)), 5);
}

#[test]
fn pads_cjk_to_exact_columns() {
    // Four wide chars = 8 columns; must not overflow a 12-column field.
    assert_eq!(visible_width(&line::fit("日本語のテキスト", 12)), 12);
}

#[test]
fn truncates_cjk_without_overflowing() {
    let fitted = line::fit("日本語", 4);
    assert_eq!(fitted, "日本");
    assert_eq!(visible_width(&fitted), 4);
}

#[test]
fn pads_when_wide_glyph_straddles_edge() {
    // A 2-column glyph cannot fit the single remaining column, so the
    // line is padded with a space rather than overflowing the border.
    let fitted = line::fit("日本", 3);
    assert_eq!(fitted, "日 ");
    assert_eq!(visible_width(&fitted), 3);
}

#[test]
fn pads_emoji_to_exact_columns() {
    assert_eq!(visible_width(&line::fit("emoji: 🚀 rocket", 20)), 20);
}

#[test]
fn ignores_ansi_escapes_when_measuring() {
    let fitted = line::fit("\x1b[31mred\x1b[0m", 6);
    assert!(fitted.starts_with("\x1b[31mred\x1b[0m"));
    assert_eq!(visible_width(&fitted), 6);
}

#[test]
fn resets_style_when_truncating_styled_text() {
    let fitted = line::fit("\x1b[31mredtext\x1b[0m", 3);
    assert!(fitted.ends_with("\x1b[0m"));
    assert_eq!(visible_width(&fitted), 3);
}
