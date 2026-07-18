use super::*;

pub(super) fn aligned(
    current: i64,
    start: i64,
    size: i64,
    view_start: i64,
    view_size: i64,
    alignment: alignment::Alignment,
) -> i64 {
    let desired = alignment::axis(start, size, view_start, view_size, alignment);
    current.saturating_add(desired.saturating_sub(view_start))
}
