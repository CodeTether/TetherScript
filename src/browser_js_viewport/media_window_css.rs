//! Standalone computed-style width-media projection.

#[path = "media_window_css_query.rs"]
mod query;

pub(super) fn apply(source: &str, width: i64) {
    let active = project(source, width);
    super::super::super::LAYOUT_CSS.with(|css| *css.borrow_mut() = active);
}

fn project(source: &str, width: i64) -> String {
    let mut output = String::new();
    let mut rest = source;
    loop {
        let lower = rest.to_ascii_lowercase();
        let Some(at) = lower.find("@media") else {
            output.push_str(rest);
            return output;
        };
        output.push_str(&rest[..at]);
        let after = &rest[at + "@media".len()..];
        let Some(open_rel) = after.find('{') else {
            output.push_str(&rest[at..]);
            return output;
        };
        let open = at + "@media".len() + open_rel;
        let Some(close) = matching_brace(rest, open) else {
            output.push_str(&rest[at..]);
            return output;
        };
        if query::matches(after[..open_rel].trim(), width) {
            output.push_str(&project(&rest[open + 1..close], width));
        }
        rest = &rest[close + 1..];
    }
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in source.bytes().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' if depth == 1 => return Some(index),
            b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}
