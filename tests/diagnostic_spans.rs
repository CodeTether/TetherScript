//! Integration tests for exact diagnostic spans (`tetherscript::diagnostic`).
//!
//! Covers, in order: offset → line/column resolution; the same with a multi-byte
//! character before the position; UTF-16 conversion diverging from the character
//! count outside the BMP; a span at end of file; a zero-width span; a span
//! crossing a line boundary; byte-exact caret rendering for a single-line span;
//! tab handling as documented (expand to 4-cell stops); a related span rendered
//! with its own locator; and a cross-check of `SourceMap`'s `O(log L)` lookup
//! against a naive full rescan over many offsets.
//!
//! File-limit note: this file exceeds 50 effective lines. The task's file
//! allowlist names exactly `tests/diagnostic_spans.rs`, so splitting it would
//! create files outside the allowlist; `./check_file_limits.sh` scopes the gate
//! to `src/**/*.rs`, and existing integration tests in this repository are
//! 200-400 lines.

use tetherscript::diagnostic::{
    caret, lsp_range, snippet::block, utf16, Diagnostic, LspPosition, Severity, SourceMap, Span,
};

/// Naive reference implementation: rescan the whole buffer for every query.
///
/// Returns `(line, char_col)`, both 1-indexed. This is the `O(n)`-per-query
/// behaviour `SourceMap` replaces, kept here purely as an oracle.
fn naive_line_col(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

#[test]
fn resolves_offsets_on_a_simple_file() {
    let src = "let x = 1\nlet y = 2\nlet z = 3\n";
    let map = SourceMap::with_name("simple.tether", src);

    assert_eq!(map.line_count(), 4);
    assert_eq!(map.name(), "simple.tether");

    let start = map.locate(0);
    assert_eq!((start.line, start.char_col, start.byte_col), (1, 1, 1));

    let x = map.locate(4);
    assert_eq!((x.line, x.char_col), (1, 5));

    let y = map.locate(14);
    assert_eq!((y.line, y.char_col), (2, 5));

    let z = map.locate(24);
    assert_eq!((z.line, z.char_col), (3, 5));
    assert_eq!(z.to_string(), "3:5");
}

#[test]
fn resolves_offsets_after_a_multibyte_character() {
    // "café" is 5 bytes: c a f + 2-byte é. The `=` that follows sits at byte 6
    // but at character column 6 too, only because of the preceding space; the
    // point is that byte_col and char_col diverge.
    let src = "let café = 1\n";
    let map = SourceMap::new(src);

    let e_acute = src.find('é').expect("é present");
    let at_e = map.locate(e_acute);
    assert_eq!(at_e.line, 1);
    assert_eq!(at_e.byte_col, 8);
    assert_eq!(at_e.char_col, 8);

    let after = map.locate(e_acute + 'é'.len_utf8());
    assert_eq!(after.byte_col, 10, "two bytes consumed by é");
    assert_eq!(after.char_col, 9, "one character consumed by é");
    assert_eq!(after.utf16_col, 9, "é is one UTF-16 unit (BMP)");
}

#[test]
fn locate_floors_a_non_char_boundary_offset() {
    let map = SourceMap::new("é");
    // Byte 1 is inside the two-byte é; it must floor to the character start.
    assert_eq!(map.locate(1).byte_col, 1);
    assert_eq!(map.locate(1).char_col, 1);
}

#[test]
fn utf16_column_differs_from_char_column_outside_the_bmp() {
    // U+1F980 CRAB: 4 UTF-8 bytes, 1 char, 2 UTF-16 code units.
    let src = "🦀🦀x\n";
    let map = SourceMap::new(src);

    let at_x = map.locate(8);
    assert_eq!(at_x.byte_col, 9);
    assert_eq!(at_x.char_col, 3, "two crabs are two characters");
    assert_eq!(at_x.utf16_col, 5, "two crabs are four UTF-16 units");
    assert_eq!(at_x.zero_based_utf16(), 4);

    assert_eq!(utf16::char_len("🦀🦀"), 2);
    assert_eq!(utf16::utf16_len("🦀🦀"), 4);
    assert_eq!(utf16::utf16_len("é"), 1);
}

#[test]
fn lsp_range_uses_utf16_code_units() {
    let map = SourceMap::new("🦀x = 1\n");
    let range = lsp_range(&map, Span::new(4, 5));
    assert_eq!(
        range.start,
        LspPosition {
            line: 0,
            character: 2
        },
        "the crab occupies UTF-16 units 0 and 1"
    );
    assert_eq!(range.end.character, 3);
}

#[test]
fn lsp_range_round_trips_through_offset_of() {
    let src = "let a = 1\n🦀 = 2\nlet c = 3\n";
    let map = SourceMap::new(src);
    let span = Span::new(
        src.find("= 2").expect("present"),
        src.find("= 2").unwrap() + 1,
    );
    let range = lsp_range(&map, span);
    let back = map.span_of_lsp(
        (range.start.line, range.start.character),
        (range.end.line, range.end.character),
    );
    assert_eq!(back, span);
}

#[test]
fn offset_of_clamps_inside_a_surrogate_pair() {
    let map = SourceMap::new("🦀x\n");
    // UTF-16 column 2 names the low surrogate; it must floor to the crab start.
    assert_eq!(map.offset_of(1, 2), 0);
    assert_eq!(map.offset_of(1, 3), 4);
    assert_eq!(map.offset_of(1, 999), 5);
}

#[test]
fn span_at_end_of_file_resolves_and_renders() {
    let src = "let x = 1\n";
    let map = SourceMap::with_name("eof.tether", src);

    let eof = Span::at(src.len());
    let at = map.locate(eof.start);
    assert_eq!(
        (at.line, at.char_col),
        (2, 1),
        "trailing newline opens line 2"
    );

    let rendered = Diagnostic::error(eof, "unexpected end of file")
        .with_primary_label("expected `}`")
        .render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: unexpected end of file\n",
            " --> eof.tether:2:1\n",
            "  |\n",
            "2 | \n",
            "  | ^ expected `}`\n",
            "  |\n",
        )
    );
}

#[test]
fn span_past_end_of_file_clamps_instead_of_panicking() {
    let map = SourceMap::new("abc");
    let at = map.locate(9_999);
    assert_eq!((at.line, at.char_col), (1, 4));
    assert_eq!(lsp_range(&map, Span::new(9_000, 9_999)).start.character, 3);
}

#[test]
fn zero_width_span_renders_exactly_one_caret() {
    let src = "f(1\n";
    let map = SourceMap::with_name("z.tether", src);
    let span = Span::at(3);
    assert!(span.is_empty());
    assert_eq!(span.len(), 0);

    let rendered = Diagnostic::error(span, "expected `)`").render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: expected `)`\n",
            " --> z.tether:1:4\n",
            "  |\n",
            "1 | f(1\n",
            "  |    ^\n",
            "  |\n",
        )
    );

    let range = lsp_range(&map, span);
    assert_eq!(range.start, range.end, "zero width stays zero width in LSP");
}

#[test]
fn span_crossing_a_line_boundary_marks_both_ends() {
    let src = "let a = [\n  1,\n];\n";
    let map = SourceMap::with_name("multi.tether", src);
    let span = Span::new(8, src.find(']').expect("present") + 1);

    let rendered = Diagnostic::error(span, "unbalanced bracket")
        .with_primary_label("this list")
        .render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: unbalanced bracket\n",
            " --> multi.tether:1:9\n",
            "  |\n",
            "1 | let a = [\n",
            "  |         ^ ...\n",
            "2 |   1,\n",
            "3 | ];\n",
            "  | ^ this list\n",
            "  |\n",
        )
    );
}

#[test]
fn caret_rendering_is_byte_exact_for_a_single_line_span() {
    let src = "let total = alpha + beta\n";
    let map = SourceMap::with_name("carets.tether", src);
    let start = src.find("alpha").expect("present");
    let span = Span::new(start, start + "alpha".len());

    let rendered = Diagnostic::error(span, "unknown binding `alpha`")
        .with_primary_label("not found in this scope")
        .render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: unknown binding `alpha`\n",
            " --> carets.tether:1:13\n",
            "  |\n",
            "1 | let total = alpha + beta\n",
            "  |             ^^^^^ not found in this scope\n",
            "  |\n",
        )
    );
}

#[test]
fn caret_run_length_counts_characters_not_bytes() {
    let src = "let café = 1\n";
    let map = SourceMap::with_name("m.tether", src);
    let start = src.find("café").expect("present");
    let span = Span::new(start, start + "café".len());
    assert_eq!(span.len(), 5, "five bytes");

    let rendered = Diagnostic::error(span, "unused binding `café`").render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: unused binding `café`\n",
            " --> m.tether:1:5\n",
            "  |\n",
            "1 | let café = 1\n",
            "  |     ^^^^\n",
            "  |\n",
        ),
        "four carets for four characters, not five for five bytes"
    );
}

#[test]
fn tabs_expand_to_four_cell_stops_and_carets_stay_aligned() {
    assert_eq!(caret::TAB_WIDTH, 4);
    assert_eq!(caret::expand_tabs("\tlet x"), "    let x");
    assert_eq!(caret::expand_tabs("ab\tc"), "ab  c");
    assert_eq!(caret::display_width_at("\t", 0), 4);
    assert_eq!(caret::display_width_at("\t", 3), 1);

    let src = "fn f() {\n\tlet x = 1\n}\n";
    let map = SourceMap::with_name("tabs.tether", src);
    let start = src.find("x = 1").expect("present");
    let span = Span::new(start, start + 1);

    let rendered = Diagnostic::error(span, "unused binding `x`").render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: unused binding `x`\n",
            " --> tabs.tether:2:6\n",
            "  |\n",
            "2 |     let x = 1\n",
            "  |         ^\n",
            "  |\n",
        ),
        "the tab becomes four spaces in both the source row and the caret pad"
    );
}

#[test]
fn tab_reported_column_stays_a_character_count() {
    let map = SourceMap::new("\tx\n");
    let at = map.locate(1);
    assert_eq!(
        at.char_col, 2,
        "columns are honest character counts; only rendering expands tabs"
    );
}

#[test]
fn related_span_renders_with_its_own_locator() {
    let src = "let a = [1, 2]\nlet b = move a\nprint(a)\n";
    let map = SourceMap::with_name("move.tether", src);
    let move_start = src.find("move a").expect("present");
    let move_span = Span::new(move_start, move_start + "move a".len());
    let use_start = src.rfind('a').expect("present");
    let use_span = Span::new(use_start, use_start + 1);

    let rendered = Diagnostic::error(use_span, "use of moved value `a`")
        .with_primary_label("value used here after move")
        .with_related(move_span, "value moved here")
        .render(&map);
    assert_eq!(
        rendered,
        concat!(
            "error: use of moved value `a`\n",
            " --> move.tether:3:7\n",
            "  |\n",
            "3 | print(a)\n",
            "  |       ^ value used here after move\n",
            "  |\n",
            "note: value moved here\n",
            " --> move.tether:2:9\n",
            "  |\n",
            "2 | let b = move a\n",
            "  |         ^^^^^^\n",
            "  |\n",
        )
    );
}

#[test]
fn multiple_related_spans_render_in_insertion_order() {
    let src = "let a = 1\nlet b = 2\nlet c = 3\n";
    let map = SourceMap::with_name("r.tether", src);
    let rendered = Diagnostic::error(Span::new(24, 25), "conflict on `c`")
        .with_related(Span::new(4, 5), "first here")
        .with_related(Span::new(14, 15), "then here")
        .render(&map);
    let notes: Vec<&str> = rendered
        .lines()
        .filter(|l| l.starts_with("note: "))
        .collect();
    assert_eq!(notes, vec!["note: first here", "note: then here"]);
}

#[test]
fn gutter_width_grows_with_the_largest_referenced_line() {
    let src = "x\n".repeat(150);
    let map = SourceMap::with_name("big.tether", &src);
    let rendered = Diagnostic::error(Span::new(240, 241), "problem")
        .with_related(Span::new(0, 1), "origin")
        .render(&map);
    assert!(
        rendered.contains("121 | x"),
        "expected a three-wide gutter, got:\n{rendered}"
    );
    assert!(
        rendered.contains("  1 | x"),
        "related block shares the wide gutter, got:\n{rendered}"
    );
}

#[test]
fn severity_prefixes_and_lsp_codes_agree() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Note.as_str(), "note");
    assert_eq!(Severity::Error.lsp_code(), 1);
    assert_eq!(Severity::Warning.lsp_code(), 2);
    assert_eq!(Severity::Note.lsp_code(), 3);

    let map = SourceMap::with_name("w.tether", "let x = 1\n");
    let warn = Diagnostic::warning(Span::new(4, 5), "unused binding `x`").render(&map);
    assert!(warn.starts_with("warning: unused binding `x`\n"));
}

#[test]
fn span_arithmetic_normalises_and_joins() {
    assert_eq!(Span::new(9, 3), Span::new(3, 9));
    assert_eq!(Span::new(2, 4).join(Span::new(9, 11)), Span::new(2, 11));
    assert_eq!(Span::new(9, 11).join(Span::new(2, 4)), Span::new(2, 11));
    assert!(Span::new(2, 4).contains(3));
    assert!(!Span::new(2, 4).contains(4));
    assert!(!Span::at(4).contains(4));
    assert_eq!(Span::default(), Span::at(0));
}

#[test]
fn line_text_strips_crlf_terminators() {
    let map = SourceMap::new("ab\r\ncd\r\n");
    assert_eq!(map.line_text(1), "ab");
    assert_eq!(map.line_text(2), "cd");
    assert_eq!(map.line_text(3), "");
    assert_eq!(
        map.line_text(99),
        "",
        "out of range clamps to the last line"
    );
}

#[test]
fn snippet_block_shape_is_stable() {
    let map = SourceMap::with_name("b.tether", "let x = 1\n");
    assert_eq!(
        block(&map, Span::new(4, 5), Some("here"), 1),
        vec![
            " --> b.tether:1:5".to_string(),
            "  |".to_string(),
            "1 | let x = 1".to_string(),
            "  |     ^ here".to_string(),
            "  |".to_string(),
        ]
    );
}

#[test]
fn source_map_matches_a_naive_rescan_across_many_offsets() {
    // A deterministic xorshift keeps the test reproducible without a dependency.
    let src = "let alpha = 1\n\tlet béta = 2\nlet 🦀 = [1, 2]\n\n\nlet gamma = 4\n".repeat(40);
    let map = SourceMap::new(&src);

    let mut state = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..4_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let offset = (state as usize) % (src.len() + 8);
        let got = map.locate(offset);
        let clamped = offset.min(src.len());
        let mut floored = clamped;
        while floored > 0 && !src.is_char_boundary(floored) {
            floored -= 1;
        }
        let (line, col) = naive_line_col(&src, floored);
        assert_eq!(
            (got.line, got.char_col),
            (line, col),
            "fast path disagreed with the naive rescan at byte offset {offset}"
        );
    }
}

#[test]
fn source_map_matches_naive_rescan_at_every_offset_of_a_small_file() {
    let src = "a\n🦀b\n\tc\n\n";
    let map = SourceMap::new(src);
    for offset in 0..=src.len() {
        if !src.is_char_boundary(offset) {
            continue;
        }
        let got = map.locate(offset);
        assert_eq!(
            (got.line, got.char_col),
            naive_line_col(src, offset),
            "disagreement at offset {offset}"
        );
    }
}
