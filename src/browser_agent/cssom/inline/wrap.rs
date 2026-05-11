//! Line breaking and inline layout algorithm.

use super::fragment::{FragmentKind, InlineFragment};
use super::line::LineBox;
use super::metrics::measure_text_width;

/// Break fragments into lines at word boundaries within the container width.
pub fn break_lines(fragments: &[InlineFragment], container_width: i64) -> Vec<LineBox> {
    let h = fragments.iter().map(|f| f.height).max().unwrap_or(16).max(1);
    layout_inline(&split_text_fragments(fragments), container_width, h)
}

/// Lay out pre-split fragments into lines of the given width and line height.
pub fn layout_inline(frags: &[InlineFragment], width: i64, line_height: i64) -> Vec<LineBox> {
    let mut lines = Vec::new();
    let mut line = LineBox::new(0, width);
    for frag in frags {
        if line.remaining_width() < frag.width && !line.is_empty() {
            lines.push(line.finalize());
            line = LineBox::new(lines.len() as i64 * line_height, width);
        }
        line.push(frag.clone());
    }
    if !line.is_empty() {
        lines.push(line.finalize());
    }
    lines
}

/// Split text fragments into word and whitespace tokens.
fn split_text_fragments(fragments: &[InlineFragment]) -> Vec<InlineFragment> {
    let mut out = Vec::new();
    for fragment in fragments {
        match &fragment.kind {
            FragmentKind::Text { content, font_size } => {
                for token in word_tokens(content) {
                    let mut frag = fragment.clone();
                    frag.width = measure_text_width(&token, *font_size);
                    frag.kind = FragmentKind::Text { content: token, font_size: *font_size };
                    out.push(frag);
                }
            }
            _ => out.push(fragment.clone()),
        }
    }
    out
}

/// Tokenize text into alternating runs of non-whitespace and whitespace.
fn word_tokens(text: &str) -> Vec<String> {
    let (mut tokens, mut buf, mut last) = (Vec::new(), String::new(), None);
    for ch in text.chars() {
        let is_space = ch.is_whitespace();
        if matches!(last, Some(space) if space != is_space) && !buf.is_empty() {
            tokens.push(std::mem::take(&mut buf));
        }
        buf.push(ch);
        last = Some(is_space);
    }
    if !buf.is_empty() {
        tokens.push(buf);
    }
    tokens
}
