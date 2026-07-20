//! WebGL 2D texture lifecycle, upload, parameters, and sampling.

use super::*;

macro_rules! texture_mod {
    ($path:literal, $name:ident) => {
        #[path = $path]
        mod $name;
    };
}

texture_mod!("webgl_pipeline_texture_active.rs", active);
texture_mod!("webgl_pipeline_texture_bind.rs", bind);
texture_mod!("webgl_pipeline_texture_binding.rs", binding);
texture_mod!("webgl_pipeline_texture_create.rs", create);
texture_mod!("webgl_pipeline_texture_delete.rs", delete);
texture_mod!("webgl_pipeline_texture_image.rs", image);
texture_mod!("webgl_pipeline_texture_image_dom.rs", image_dom);
texture_mod!("webgl_pipeline_texture_image_data.rs", image_data);
texture_mod!("webgl_pipeline_texture_image_validation.rs", image_validation);
texture_mod!("webgl_pipeline_texture_named_constants.rs", named_constants);
texture_mod!("webgl_pipeline_texture_parameter.rs", parameter);
texture_mod!("webgl_pipeline_texture_pixel_store.rs", pixel_store);
texture_mod!("webgl_pipeline_texture_pixel_transform.rs", pixel_transform);
texture_mod!("webgl_pipeline_texture_pixels.rs", pixels);
texture_mod!("webgl_pipeline_texture_sample.rs", sample);
texture_mod!("webgl_pipeline_texture_sample_coordinate.rs", sample_coordinate);
texture_mod!("webgl_pipeline_texture_sample_linear.rs", sample_linear);
texture_mod!("webgl_pipeline_texture_sub_image.rs", sub_image);
texture_mod!("webgl_pipeline_texture_sub_image_dom.rs", sub_image_dom);
texture_mod!("webgl_pipeline_texture_sub_write.rs", sub_write);

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    create::install(obj, handle.clone(), version);
    bind::install(obj, handle.clone(), version);
    active::install(obj, handle.clone(), version);
    image::install(obj, handle.clone(), version);
    sub_image::install(obj, handle.clone(), version);
    parameter::install(obj, handle.clone(), version);
    pixel_store::install(obj, handle.clone(), version);
    delete::install(obj, handle, version);
}

pub(super) fn install_constants(obj: &mut HashMap<String, JsValue>) {
    named_constants::install(obj);
}

pub(super) fn sample(
    texture: &texture_state::Texture,
    uv: [f64; 2],
    filter: u32,
) -> [u8; 4] {
    sample::color(texture, uv, filter)
}

pub(super) fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_OPERATION);
}