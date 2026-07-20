//! `ImageData` source overload for complete level-zero texture uploads.

use super::*;

pub(super) fn upload(state: &mut WebGlState, args: &[JsValue]) {
    if !binding::target(state, args.first())
        || !image_validation::level(state, args.get(1))
        || !image_validation::format(state, args.get(2), args.get(3), args.get(4))
    {
        return;
    }
    let Some((width, height, pixels)) = image_data::decode(state, args.get(5)) else {
        return;
    };
    let Some(texture) = binding::get_mut(state) else {
        return;
    };
    texture.width = width;
    texture.height = height;
    texture.pixels = pixels;
}
