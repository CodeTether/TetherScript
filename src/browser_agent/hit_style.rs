//! Style helpers for action hit testing.

use crate::browser::LayoutBox;
use crate::browser_agent::action::BoundingBox;

#[path = "hit_z.rs"]
mod z;

pub(crate) use z::z_index;

const NON_RENDERED: [&str; 9] = [
    "base", "head", "link", "meta", "noscript", "script", "style", "template", "title",
];

pub(crate) fn bounds_for(layout: &LayoutBox) -> BoundingBox {
    BoundingBox {
        x: layout.x,
        y: layout.y,
        width: layout.width,
        height: layout.height,
    }
}

pub(crate) fn pointer_enabled(layout: &LayoutBox) -> bool {
    !layout
        .tag
        .as_deref()
        .is_some_and(|tag| NON_RENDERED.contains(&tag))
        && visible(layout)
        && !layout
            .styles
            .get("pointer-events")
            .is_some_and(|value| value.eq_ignore_ascii_case("none"))
}

fn visible(layout: &LayoutBox) -> bool {
    !layout
        .styles
        .get("visibility")
        .is_some_and(|value| matches_hidden(value))
}

fn matches_hidden(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "hidden" | "collapse"
    )
}
