use super::*;

pub(super) fn point_visible(handle: &DomHandle, x: i64, y: i64) -> bool {
    for depth in 1..handle.path.len() {
        let ancestor = DomHandle {
            root: handle.root.clone(),
            path: handle.path[..depth].to_vec(),
        };
        let geometry = geometry::measure(&ancestor);
        let (offset_x, offset_y) = state::ancestor_offset(&ancestor);
        let left = geometry.x - offset_x + geometry.client_left;
        let top = geometry.y - offset_y + geometry.client_top;
        if geometry.scrollable_x && (x < left || x >= left + geometry.client_width) {
            return false;
        }
        if geometry.scrollable_y && (y < top || y >= top + geometry.client_height) {
            return false;
        }
    }
    true
}
