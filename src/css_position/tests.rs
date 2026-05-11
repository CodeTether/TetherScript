use super::*;

fn el(position: PositionType) -> PositionedElement<usize> {
    PositionedElement {
        element: 0, parent: None, position,
        offsets: Edges::default(), z_index: None,
        normal_x: 10.0, normal_y: 20.0, width: 100.0, height: 50.0,
        computed_x: 10.0, computed_y: 20.0,
        margin: BoxEdges::default(), padding: BoxEdges::default(), border: BoxEdges::default(),
    }
}

#[test]
fn relative_offsets_from_normal_position() {
    let mut e = el(PositionType::Relative);
    e.offsets.left = Some(5.0);
    e.offsets.top = Some(7.0);
    RelativePositioner::compute(&mut e);
    assert_eq!((e.computed_x, e.computed_y), (15.0, 27.0));
}

#[test]
fn absolute_uses_containing_block_offsets() {
    let mut e = el(PositionType::Absolute);
    e.offsets.left = Some(30.0);
    e.offsets.top = Some(40.0);
    let cb = ContainingBlock { rect: Rect { x: 100.0, y: 200.0, width: 300.0, height: 300.0 } };
    AbsolutePositioner::compute(&mut e, cb);
    assert_eq!((e.computed_x, e.computed_y), (130.0, 240.0));
}

#[test]
fn fixed_uses_viewport() {
    let mut e = el(PositionType::Fixed);
    e.offsets.right = Some(10.0);
    e.offsets.bottom = Some(20.0);
    FixedPositioner::compute(&mut e, ContainingBlock::viewport(800.0, 600.0));
    assert_eq!((e.computed_x, e.computed_y), (690.0, 530.0));
}

#[test]
fn z_index_orders_correctly() {
    let mut a = el(PositionType::Absolute);
    a.z_index = Some(2);
    let b = el(PositionType::Static);
    let mut c = el(PositionType::Relative);
    c.z_index = Some(-1);
    let order = ZIndexResolver::paint_order(&[a, b, c]);
    assert_eq!(order.iter().map(|r| r.index).collect::<Vec<_>>(), vec![2, 1, 0]);
}

#[test]
fn layout_computes_mixed_positions() {
    let mut abs = el(PositionType::Absolute);
    abs.offsets.left = Some(1.0);
    abs.offsets.top = Some(2.0);
    let rel = {
        let mut r = el(PositionType::Relative);
        r.offsets.left = Some(3.0);
        r
    };
    let mut layout = PositionedLayout::new(vec![abs, rel], 500.0, 400.0);
    layout.compute();
    assert_eq!((layout.elements[0].computed_x, layout.elements[0].computed_y), (1.0, 2.0));
    assert_eq!(layout.elements[1].computed_x, 13.0);
}
