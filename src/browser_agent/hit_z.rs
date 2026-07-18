//! Z-index extraction for action hit testing.

use crate::browser::LayoutBox;

pub(crate) fn z_index(layout: &LayoutBox) -> i64 {
    px(layout.styles.get("z-index")).unwrap_or(0)
}

fn px(value: Option<&String>) -> Option<i64> {
    let value = value?.trim();
    value.strip_suffix("px").unwrap_or(value).parse().ok()
}
