//! Resolution of sampled texture coordinates, unit, and bound image.

use super::*;

pub(super) fn resolve(
    state: &mut WebGlState,
    program: &shader_state::Program,
) -> Option<Resolved> {
    let shader_state::ColorSource::Texture {
        sampler,
        coordinates,
    } = &program.color
    else {
        return Some((None, None));
    };
    let unit = *program.samplers.get(sampler).unwrap_or(&0);
    if !(0..constants::MAX_TEXTURE_UNITS as i32).contains(&unit) {
        draw::invalid(state);
        return None;
    }
    let Some(texture) = state.pipeline.texture_bindings.units[unit as usize]
        .and_then(|id| state.pipeline.textures.get(&id))
        .filter(|texture| !texture.deleted)
        .cloned()
    else {
        draw::invalid(state);
        return None;
    };
    let Some(location) = program.attributes.get(coordinates).copied() else {
        draw::invalid(state);
        return None;
    };
    Some((Some(input::resolve(state, location)?), Some(texture)))
}