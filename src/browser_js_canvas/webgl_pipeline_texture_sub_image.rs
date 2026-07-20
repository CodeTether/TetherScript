//! Bounds-checked level-zero RGBA texture sub-image upload.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "texSubImage2D".into(),
        native("WebGLRenderingContext.texSubImage2D", Some(7), move |args| {
            webgl_store::mutate(&handle, version, |state| upload(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn upload(state: &mut WebGlState, args: &[JsValue]) {
    if args.len() < 9 {
        sub_image_dom::upload(state, args);
        return;
    }
    if !binding::target(state, args.first()) || !image_validation::level(state, args.get(1)) {
        return;
    }
    if !image_validation::format(state, args.get(6), args.get(6), args.get(7)) {
        return;
    }
    let x = webgl_values::i64_value(args.get(2));
    let y = webgl_values::i64_value(args.get(3));
    let Some((width, height)) = image_validation::size(state, args.get(4), args.get(5)) else {
        return;
    };
    if x < 0 || y < 0 {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return;
    }
    let pixels = match pixels::decode(
        args.get(8), width, height, &state.pipeline.texture_bindings, false,
    ) {
        Ok(pixels) => pixels,
        Err(error) => {
            pixels::record(state, error);
            return;
        }
    };
    sub_write::write(state, x as usize, y as usize, width, height, &pixels);
}
