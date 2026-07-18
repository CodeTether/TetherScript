use super::*;

pub(super) fn children(
    children: &[browser::LayoutBox],
    mut right: i64,
    mut bottom: i64,
) -> (i64, i64) {
    for child in children {
        right = right.max(child.x.saturating_add(child.width));
        bottom = bottom.max(child.y.saturating_add(child.height));
        (right, bottom) = self::children(&child.children, right, bottom);
    }
    (right, bottom)
}
