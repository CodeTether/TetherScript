use super::*;

pub(super) fn scrollable(layout: &browser::LayoutBox, axis: &str) -> bool {
    if layout.tag.as_deref() == Some("textarea") {
        return true;
    }
    let axis_name = format!("overflow-{axis}");
    layout
        .styles
        .get(&axis_name)
        .or_else(|| layout.styles.get("overflow"))
        .is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "auto" | "scroll" | "hidden"
            )
        })
}
