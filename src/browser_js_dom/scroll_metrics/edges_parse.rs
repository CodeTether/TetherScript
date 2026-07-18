pub(super) fn box_values(value: Option<&String>) -> Option<[i64; 4]> {
    let values = value?
        .split_whitespace()
        .filter_map(part)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [all] => Some([*all, *all, *all, *all]),
        [vertical, horizontal] => Some([*vertical, *horizontal, *vertical, *horizontal]),
        [top, horizontal, bottom] => Some([*top, *horizontal, *bottom, *horizontal]),
        [top, right, bottom, left, ..] => Some([*top, *right, *bottom, *left]),
        _ => None,
    }
}

pub(super) fn part(value: &str) -> Option<i64> {
    let value = value.trim();
    value
        .strip_suffix("px")
        .unwrap_or(value)
        .trim()
        .parse()
        .ok()
}
