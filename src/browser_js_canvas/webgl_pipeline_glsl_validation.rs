//! Structural validation for the supported GLSL ES subset.

use super::super::constants;

pub(crate) fn validate(kind: u32, source: &str) -> Result<(), String> {
    if source.trim().is_empty() {
        return Err("ERROR: 0:1: shader source is empty".into());
    }
    if !balanced(source) {
        return Err("ERROR: 0:1: unbalanced shader braces".into());
    }
    if !source.contains("void main") && !source.contains("void\nmain") {
        return Err("ERROR: 0:1: shader must define void main".into());
    }
    if kind == constants::VERTEX_SHADER && !source.contains("gl_Position") {
        return Err("ERROR: 0:1: vertex shader must write gl_Position".into());
    }
    if kind == constants::FRAGMENT_SHADER
        && !source.contains("gl_FragColor")
        && !source.contains("out vec4")
    {
        return Err("ERROR: 0:1: fragment shader must declare a color output".into());
    }
    Ok(())
}

fn balanced(source: &str) -> bool {
    let mut depth = 0_i32;
    for byte in source.bytes() {
        match byte {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
    }
    depth == 0
}
