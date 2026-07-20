//! `ImageData` source overload for level-zero texture sub-updates.

use super::*;

pub(super) fn upload(state: &mut WebGlState, args: &[JsValue]) {
    if !binding::target(state, args.first())
        || !image_validation::level(state, args.get(1))
        || !image_validation::format(state, args.get(4), args.get(4), args.get(5))
    {
        return;
    }
    let x = webgl_values::i64_value(args.get(2));
    let y = webgl_values::i64_value(args.get(3));
    if x < 0 || y < 0 {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return;
    }
    let Some((width, height, pixels)) = image_data::decode(state, args.get(6)) else {
        return;
    };
    sub_write::write(state, x as usize, y as usize, width, height, &pixels);
}
