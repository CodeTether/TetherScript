pub(super) fn matches(query: &str, width: i64) -> bool {
    query.split(',').any(|part| matches_part(part, width))
}

fn matches_part(part: &str, width: i64) -> bool {
    let raw = part.trim().to_ascii_lowercase();
    let raw = raw.strip_prefix("only ").unwrap_or(&raw);
    if let Some(rest) = raw.strip_prefix("not ") {
        return !matches_part(rest, width);
    }
    raw.split("and").all(|item| matches_one(item, width))
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
        "width" => px(value).is_some_and(|limit| width == limit),
        "min-width" => px(value).is_some_and(|limit| width >= limit),
        "max-width" => px(value).is_some_and(|limit| width <= limit),
        "prefers-color-scheme" => value.trim() == "light",
        "prefers-reduced-motion" => value.trim() == "no-preference",
        "forced-colors" => value.trim() == "none",
        _ => false,
    }
}

fn px(value: &str) -> Option<i64> {
    value.trim().trim_end_matches("px").trim().parse().ok()
}
