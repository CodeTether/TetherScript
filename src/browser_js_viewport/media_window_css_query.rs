//! Width media-condition matching for standalone computed styles.

pub(super) fn matches(query: &str, width: i64) -> bool {
    query.split(',').any(|part| {
        part.split("and")
            .all(|condition| matches_one(condition, width))
    })
}

fn matches_one(raw: &str, width: i64) -> bool {
    let item = raw
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    if matches!(item, "" | "all" | "screen") {
        return true;
    }
    let Some((name, value)) = item.split_once(':') else {
        return false;
    };
    match name.trim() {
        "width" => pixels(value).is_some_and(|limit| width == limit),
        "min-width" => pixels(value).is_some_and(|limit| width >= limit),
        "max-width" => pixels(value).is_some_and(|limit| width <= limit),
        _ => false,
    }
}

fn pixels(value: &str) -> Option<i64> {
    value.trim().trim_end_matches("px").trim().parse().ok()
}
