use super::*;

pub(super) fn current_handle(object: &JsObjectRef, fallback: &DomHandle) -> DomHandle {
    dom_handle_from_value(&JsValue::Object(object.clone())).unwrap_or_else(|| fallback.clone())
}

pub(in crate::browser_js) fn reset() {
    state::reset();
}

pub(in crate::browser_js) fn rekey(old: &str, new: &str) {
    state::rekey(old, new);
}

pub(in crate::browser_js) fn scrolled_rect(
    handle: &DomHandle,
    layout: &browser::LayoutBox,
) -> (i64, i64, i64, i64) {
    geometry::scrolled_rect(handle, layout)
}

pub(in crate::browser_js) fn offset_for(handle: &DomHandle) -> (i64, i64) {
    state::ancestor_offset(handle)
}

pub(in crate::browser_js) fn point_visible(handle: &DomHandle, x: i64, y: i64) -> bool {
    visibility::point_visible(handle, x, y)
}
