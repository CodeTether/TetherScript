//! `putImageData` dirty rectangle clipping.

use super::*;

pub(super) struct Region {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

pub(super) fn region(args: &[JsValue], width: usize, height: usize) -> Region {
    if args.len() < 7 {
        return Region {
            x0: 0,
            y0: 0,
            x1: width,
            y1: height,
        };
    }
    let (x0, x1) = axis(
        super::geometry::i64_value(args.get(3)),
        super::geometry::i64_value(args.get(5)),
        width,
    );
    let (y0, y1) = axis(
        super::geometry::i64_value(args.get(4)),
        super::geometry::i64_value(args.get(6)),
        height,
    );
    Region { x0, y0, x1, y1 }
}

fn axis(origin: i64, extent: i64, bound: usize) -> (usize, usize) {
    let end = origin.saturating_add(extent);
    let (start, end) = if extent < 0 {
        (end, origin)
    } else {
        (origin, end)
    };
    (
        start.clamp(0, bound as i64) as usize,
        end.clamp(0, bound as i64) as usize,
    )
}
