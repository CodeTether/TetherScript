//! Texture-target validation and active-unit resource lookup.

use super::*;

pub(super) fn target(state: &mut WebGlState, value: Option<&JsValue>) -> bool {
    if webgl_values::u32_value(value) == constants::TEXTURE_2D {
        return true;
    }
    webgl_error::record(state, webgl_constants::INVALID_ENUM);
    false
}

pub(super) fn id(state: &WebGlState) -> Option<u32> {
    state.pipeline.texture_bindings.units[state.pipeline.texture_bindings.active]
}

pub(super) fn get(state: &mut WebGlState) -> Option<texture_state::Texture> {
    let Some(texture) = id(state)
        .and_then(|id| state.pipeline.textures.get(&id))
        .filter(|texture| !texture.deleted)
        .cloned()
    else {
        texture::invalid(state);
        return None;
    };
    Some(texture)
}

pub(super) fn get_mut(state: &mut WebGlState) -> Option<&mut texture_state::Texture> {
    let Some(id) = id(state) else {
        texture::invalid(state);
        return None;
    };
    state.pipeline.textures.get_mut(&id)
}
