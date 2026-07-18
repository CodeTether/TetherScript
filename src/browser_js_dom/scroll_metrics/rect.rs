use super::*;

pub(super) fn visible(handle: &DomHandle) -> (i64, i64, i64, i64) {
    let geometry = geometry::measure(handle);
    let (left, top) = state::ancestor_offset(handle);
    (
        geometry.x - left,
        geometry.y - top,
        geometry.width,
        geometry.height,
    )
}
