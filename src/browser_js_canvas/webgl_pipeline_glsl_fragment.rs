//! Fragment color extraction for the software GLSL subset.

use super::super::shader_state::ColorSource;

pub(crate) fn color(
    source: &str,
    uniforms: &[String],
    samplers: &[String],
) -> Result<ColorSource, String> {
    if let Some(texture) = super::texture::color(source, samplers) {
        return Ok(texture);
    }
    if let Some(values) = constant(source) {
        return Ok(ColorSource::Constant(values));
    }
    if let Some(name) = assigned_name(source) {
        if uniforms.contains(&name) {
            return Ok(ColorSource::Uniform(name));
        }
    }
    Err("link: fragment output must be a constant, uniform, or sampled RGBA value".into())
}

fn constant(source: &str) -> Option<[f64; 4]> {
    let start = source.rfind("vec4")?;
    let args = source[start..].split_once('(')?.1.split_once(')')?.0;
    let values: Vec<f64> = args
        .split(',')
        .map(str::trim)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match values.as_slice() {
        [value] => Some([*value; 4]),
        [red, green, blue, alpha] => Some([*red, *green, *blue, *alpha]),
        _ => None,
    }
}

fn assigned_name(source: &str) -> Option<String> {
    let output = source.find("gl_FragColor")?;
    let rhs = source[output..]
        .split_once('=')?
        .1
        .split_once(';')?
        .0
        .trim();
    (!rhs.is_empty()).then(|| rhs.to_string())
}
