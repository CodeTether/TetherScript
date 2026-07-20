//! GLSL attribute and uniform declaration reflection.

pub(crate) fn attributes(source: &str) -> Vec<String> {
    declarations(source, &["attribute", "in"], &["float", "vec2", "vec3", "vec4"])
}

pub(crate) fn uniforms(source: &str) -> Vec<String> {
    declarations(source, &["uniform"], &["float", "vec2", "vec3", "vec4"])
}

pub(crate) fn samplers(source: &str) -> Vec<String> {
    declarations(source, &["uniform"], &["sampler2D"])
}

fn declarations(source: &str, qualifiers: &[&str], types: &[&str]) -> Vec<String> {
    let tokens: Vec<&str> = source
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|token| !token.is_empty())
        .collect();
    let mut names = Vec::new();
    for window in tokens.windows(3) {
        if qualifiers.contains(&window[0]) && types.contains(&window[1]) {
            let name = window[2].to_string();
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}
