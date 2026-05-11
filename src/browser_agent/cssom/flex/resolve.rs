//! Axis resolution and flex item collection.

use std::collections::HashMap;
use super::{parse, types::*};

pub fn is_row(d: FlexDirection) -> bool {
    matches!(d, FlexDirection::Row | FlexDirection::RowReverse)
}

pub fn is_reverse(d: FlexDirection) -> bool {
    matches!(d, FlexDirection::RowReverse | FlexDirection::ColumnReverse)
}

pub fn collect(styles: &[HashMap<String, String>], sizes: &[(i64, i64)], dir: FlexDirection) -> Vec<FlexItem> {
    let row = is_row(dir);
    styles.iter().enumerate().map(|(i, st)| {
        let (w, h) = sizes.get(i).copied().unwrap_or((0, 0));
        let basis = parse::len(st, "flex-basis")
            .or_else(|| if row { parse::len(st, "width") } else { parse::len(st, "height") });
        FlexItem {
            flex_grow: parse::num(st, "flex-grow", 0.0),
            flex_shrink: parse::num(st, "flex-shrink", 1.0),
            flex_basis: basis,
            min_size: parse::len(st, if row { "min-width" } else { "min-height" }).unwrap_or(0),
            max_size: parse::len(st, if row { "max-width" } else { "max-height" }).unwrap_or(i64::MAX / 4),
            x: 0, y: 0, width: w, height: h,
        }
    }).collect()
}

pub fn main_size(it: &FlexItem, row: bool) -> i64 { if row { it.width } else { it.height } }
pub fn cross_size(it: &FlexItem, row: bool) -> i64 { if row { it.height } else { it.width } }
pub fn set_main(it: &mut FlexItem, row: bool, v: i64) { if row { it.width = v } else { it.height = v } }
pub fn set_cross(it: &mut FlexItem, row: bool, v: i64) { if row { it.height = v } else { it.width = v } }
pub fn set_main_pos(it: &mut FlexItem, row: bool, v: i64) { if row { it.x = v } else { it.y = v } }
pub fn set_cross_pos(it: &mut FlexItem, row: bool, v: i64) { if row { it.y = v } else { it.x = v } }
