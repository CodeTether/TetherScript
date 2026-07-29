//! Tests for per-character and per-string terminal width measurement.

use super::measure::{char_width, str_width};

#[test]
fn ascii_is_one_column_each() {
    assert_eq!(char_width('a'), 1);
    assert_eq!(str_width("hello"), 5);
}

#[test]
fn cjk_is_two_columns_each() {
    assert_eq!(char_width('日'), 2);
    assert_eq!(str_width("日本語"), 6);
}

#[test]
fn combining_marks_are_zero_width() {
    assert_eq!(char_width('\u{0301}'), 0);
    // "e" plus combining acute accent renders in one cell.
    assert_eq!(str_width("e\u{0301}"), 1);
}

#[test]
fn variation_selector_is_zero_width() {
    assert_eq!(char_width('\u{FE0F}'), 0);
}

#[test]
fn skin_tone_modifier_is_wide() {
    // Fitzpatrick modifiers are Sk + East Asian Wide, not combining
    // marks, so the per-codepoint model gives them two columns. Terminals
    // disagree on collapsing full ZWJ emoji sequences; this module
    // deliberately measures one codepoint at a time.
    assert_eq!(char_width('\u{1F3FB}'), 2);
}

#[test]
fn emoji_are_two_columns() {
    assert_eq!(char_width('🚀'), 2);
    // Symbols promoted to Wide by emoji presentation.
    assert_eq!(char_width('⭐'), 2);
    assert_eq!(char_width('⌚'), 2);
    assert_eq!(char_width('✅'), 2);
}

#[test]
fn fullwidth_forms_are_two_columns() {
    assert_eq!(char_width('Ａ'), 2);
}

#[test]
fn narrow_symbols_stay_one_column() {
    // Box-drawing and arrows are Neutral/Ambiguous, not Wide.
    assert_eq!(char_width('─'), 1);
    assert_eq!(char_width('│'), 1);
    assert_eq!(char_width('→'), 1);
}
