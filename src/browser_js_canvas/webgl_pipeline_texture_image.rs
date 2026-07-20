//! Complete level-zero RGBA texture allocation and upload.

use super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "texImage2D".into(),
        native("WebGLRenderingContext.texImage2D", Some(6), move |args| {
            webgl_store::mutate(&handle, version, |state| upload(state, args));
            Ok(JsValue::Undefined)
        }),
    );
}

fn upload(state: &mut WebGlState, args: &[JsValue]) {
    if args.len() < 9 {
        image_dom::upload(state, args);
        return;
    }
    if !binding::target(state, args.first())
        || !image_validation::level(state, args.get(1))
        || !image_validation::format(state, args.get(2), args.get(6), args.get(7))
    {
        return;
    }
    if webgl_values::i64_value(args.get(5)) != 0 {
        webgl_error::record(state, webgl_constants::INVALID_VALUE);
        return;
    }
    let Some((width, height)) = image_validation::size(state, args.get(3), args.get(4)) else {
        return;
    };
    let pixels = match pixels::decode(
        args.get(8), width, height, &state.pipeline.texture_bindings, true,
    ) {
        Ok(pixels) => pixels,
        Err(error) => {
            pixels::record(state, error);
            return;
        }
    };
    let Some(texture) = binding::get_mut(state) else {
        return;
    };
    texture.width = width;
    texture.height = height;
    texture.pixels = pixels;
}
