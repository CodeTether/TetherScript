use super::*;

fn item(w: f32, h: f32) -> FlexItem {
    FlexItem {
        content_size: Size { width: w, height: h },
        desired_size: Size { width: w, height: h },
        ..FlexItem::default()
    }
}

fn cons(w: f32, h: f32) -> FlexConstraints {
    FlexConstraints {
        available: Size { width: w, height: h },
    }
}

#[test]
fn row_grow_distributes_free_space() {
    let c = FlexContainer::default();
    let mut a = item(50.0, 10.0);
    let mut b = item(50.0, 10.0);
    a.flex_grow = 1.0;
    b.flex_grow = 3.0;

    let r = layout_flex(&c, &[a, b], cons(200.0, 20.0));
    assert_eq!(r[0].rect.width, 75.0);
    assert_eq!(r[1].rect.width, 125.0);
    assert_eq!(r[1].rect.x, 75.0);
}

#[test]
fn row_shrink_uses_flex_shrink() {
    let c = FlexContainer::default();
    let a = item(100.0, 10.0);
    let b = item(100.0, 10.0);

    let r = layout_flex(&c, &[a, b], cons(100.0, 20.0));
    assert_eq!(r[0].rect.width, 50.0);
    assert_eq!(r[1].rect.width, 50.0);
}

#[test]
fn flex_basis_zero_ignores_content_then_grows() {
    let c = FlexContainer::default();
    let mut a = item(100.0, 10.0);
    let mut b = item(100.0, 10.0);
    a.flex_basis = FlexBasis::Zero;
    b.flex_basis = FlexBasis::Zero;
    a.flex_grow = 1.0;
    b.flex_grow = 1.0;

    let r = layout_flex(&c, &[a, b], cons(120.0, 20.0));
    assert_eq!(r[0].rect.width, 60.0);
    assert_eq!(r[1].rect.width, 60.0);
}

#[test]
fn justify_center_offsets_items() {
    let mut c = FlexContainer::default();
    c.justify_content = JustifyContent::Center;

    let r = layout_flex(&c, &[item(20.0, 10.0)], cons(100.0, 20.0));
    assert_eq!(r[0].rect.x, 40.0);
}

#[test]
fn wrap_creates_multiple_lines() {
    let mut c = FlexContainer::default();
    c.wrap = FlexWrap::Wrap;
    c.gap.row_gap = 5.0;

    let r = layout_flex(
        &c,
        &[item(60.0, 10.0), item(60.0, 20.0)],
        cons(100.0, 100.0),
    );
    assert_eq!(r[0].rect.y, 0.0);
    assert_eq!(r[1].rect.y, 15.0);
}

#[test]
fn column_direction_uses_vertical_main_axis() {
    let mut c = FlexContainer::default();
    c.direction = FlexDirection::Column;

    let r = layout_flex(
        &c,
        &[item(10.0, 20.0), item(10.0, 30.0)],
        cons(100.0, 100.0),
    );
    assert_eq!(r[0].rect.y, 0.0);
    assert_eq!(r[1].rect.y, 20.0);
    assert_eq!(r[1].rect.height, 30.0);
}

#[test]
fn row_reverse_places_first_item_at_end() {
    let mut c = FlexContainer::default();
    c.direction = FlexDirection::RowReverse;

    let r = layout_flex(
        &c,
        &[item(20.0, 10.0), item(20.0, 10.0)],
        cons(100.0, 20.0),
    );
    assert_eq!(r[0].rect.x, 80.0);
    assert_eq!(r[1].rect.x, 60.0);
}

#[test]
fn order_changes_visual_sequence_but_output_sorted_by_index() {
    let c = FlexContainer::default();
    let mut a = item(10.0, 10.0);
    let mut b = item(10.0, 10.0);
    a.order = 2;
    b.order = 1;

    let r = layout_flex(&c, &[a, b], cons(100.0, 20.0));
    assert_eq!(r[0].index, 0);
    assert_eq!(r[0].rect.x, 10.0);
    assert_eq!(r[1].index, 1);
    assert_eq!(r[1].rect.x, 0.0);
}
