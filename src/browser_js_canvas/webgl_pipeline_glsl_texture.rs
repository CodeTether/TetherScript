//! Reflection of direct `texture2D` and GLSL ES 3 `texture` output calls.

use super::super::shader_state::ColorSource;

pub(crate) fn color(source: &str, samplers: &[String]) -> Option<ColorSource> {
    let compact: String = source.chars().filter(|ch| !ch.is_whitespace()).collect();
    let arguments = ["texture2D(", "texture("]
        .iter()
        .find_map(|call| arguments(&compact, call))?;
    let (sampler, coordinates) = arguments.split_once(',')?;
    if !samplers.iter().any(|name| name == sampler) || coordinates.is_empty() {
        return None;
    }
    Some(ColorSource::Texture {
        sampler: sampler.into(),
        coordinates: coordinates.into(),
    })
}

fn arguments<'a>(source: &'a str, call: &str) -> Option<&'a str> {
    let start = source.find(call)? + call.len();
    source[start..].split_once(')').map(|(arguments, _)| arguments)
}
