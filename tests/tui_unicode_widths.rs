//! Integration test: wide and zero-width glyphs keep panel borders aligned.
//!
//! Regression guard for the `tui_render` fitting path, which measured
//! character counts instead of terminal columns. CJK and emoji rows used
//! to overflow the right border by one to eight cells.

use std::process::Command;

/// Codepoint ranges treated as zero-width when measuring expected output.
fn is_zero_width(ch: char) -> bool {
    matches!(ch as u32, 0x0300..=0x036F | 0x200B..=0x200F | 0xFE00..=0xFE0F)
}

/// Codepoint ranges treated as double-width when measuring expected output.
fn is_wide(ch: char) -> bool {
    matches!(ch as u32,
        0x1100..=0x115F | 0x2B50..=0x2B50 | 0x2E80..=0x303E | 0x3041..=0x33FF
        | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7A3 | 0xFF00..=0xFF60
        | 0x1F300..=0x1F64F | 0x1F680..=0x1F6FF | 0x1F900..=0x1F9FF)
}

/// Independent column measurement, deliberately not sharing crate internals.
fn columns(line: &str) -> usize {
    line.chars()
        .filter(|ch| !is_zero_width(*ch))
        .map(|ch| if is_wide(ch) { 2 } else { 1 })
        .sum()
}

#[test]
fn unicode_panel_rows_are_column_aligned() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/tui_unicode_widths.tether"])
        .output()
        .expect("tetherscript binary should run");
    assert!(
        output.status.success(),
        "example failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 8, "expected an 8-row panel, got {lines:?}");

    // The example declares width 44; every row must occupy exactly that.
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(
            columns(line),
            44,
            "row {} is {} columns, not 44: {line:?}",
            index + 1,
            columns(line)
        );
    }
}

#[test]
fn unicode_panel_truncates_wide_text_without_overflow() {
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .args(["run", "examples/tui_unicode_widths.tether"])
        .output()
        .expect("tetherscript binary should run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // The long CJK row is cut to fit, so it must not keep every glyph.
    let row = stdout
        .lines()
        .find(|line| line.contains("truncated:"))
        .expect("truncated row should be present");
    assert!(
        !row.contains("日本語日本語日本語日本語日本語日本語"),
        "row was not truncated: {row:?}"
    );
    assert_eq!(columns(row), 44);
}
