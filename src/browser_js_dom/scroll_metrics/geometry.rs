use super::*;

pub(super) use geometry_model::Geometry;

pub(super) fn measure(handle: &DomHandle) -> Geometry {
    let layout = layout_for_handle(handle);
    let Some(item) = browser::find_layout_box_at_path(&layout, &handle.path) else {
        return Geometry::default();
    };
    let [top, right, bottom, left] = edges::border(handle);
    let client_width = (item.width - left - right).max(0);
    let client_height = (item.height - top - bottom).max(0);
    let origin_x = item.x.saturating_add(left);
    let origin_y = item.y.saturating_add(top);
    let initial_right = origin_x.saturating_add(client_width);
    let initial_bottom = origin_y.saturating_add(client_height);
    let (content_right, content_bottom) =
        extent::children(&item.children, initial_right, initial_bottom);
    Geometry {
        x: item.x,
        y: item.y,
        width: item.width,
        height: item.height,
        client_left: left,
        client_top: top,
        client_width,
        client_height,
        scroll_width: content_right.saturating_sub(origin_x).max(client_width),
        scroll_height: content_bottom.saturating_sub(origin_y).max(client_height),
        scrollable_x: overflow::scrollable(item, "x"),
        scrollable_y: overflow::scrollable(item, "y"),
    }
}

pub(super) fn scrolled_rect(
    handle: &DomHandle,
    layout: &browser::LayoutBox,
) -> (i64, i64, i64, i64) {
    let (x, y) = state::ancestor_offset(handle);
    let viewport = window_host::scroll::metrics();
    (
        layout.x - x - viewport.x,
        layout.y - y - viewport.y,
        layout.width,
        layout.height,
    )
}
