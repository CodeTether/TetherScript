//! Link-time attribute, uniform, and fragment-output reflection.

use super::*;

pub(super) type Reflection = (
    HashMap<String, u32>,
    HashMap<String, [f64; 4]>,
    HashMap<String, i32>,
    shader_state::ColorSource,
);

pub(super) fn reflect(state: &WebGlState, id: u32) -> Result<Reflection, String> {
    let program = state
        .pipeline
        .programs
        .get(&id)
        .ok_or("link: invalid program")?;
    let vertex = shader(state, program.vertex, "vertex")?;
    let fragment = shader(state, program.fragment, "fragment")?;
    let attribute_names = glsl::attributes(&vertex.source);
    let attributes = attribute_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index as u32))
        .collect();
    let names = glsl::uniforms(&fragment.source);
    let uniforms = names.iter().map(|name| (name.clone(), [0.0; 4])).collect();
    let sampler_names = glsl::samplers(&fragment.source);
    let samplers = sampler_names.iter().map(|name| (name.clone(), 0)).collect();
    let mut color = glsl::color(&fragment.source, &names, &sampler_names)?;
    if let shader_state::ColorSource::Texture { coordinates, .. } = &mut color {
        *coordinates = glsl::varying_attribute(&vertex.source, coordinates, &attribute_names)
            .ok_or("link: sampled texture coordinates must come from a vertex attribute")?;
    }
    Ok((attributes, uniforms, samplers, color))
}

fn shader<'a>(
    state: &'a WebGlState,
    id: Option<u32>,
    stage: &str,
) -> Result<&'a shader_state::Shader, String> {
    id.and_then(|id| state.pipeline.shaders.get(&id))
        .filter(|shader| shader.compiled)
        .ok_or_else(|| format!("link: compiled {stage} shader required"))
}
