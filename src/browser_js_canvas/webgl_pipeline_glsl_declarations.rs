//! GLSL attribute and uniform declaration reflection.

pub(crate) fn attributes(source: &str) -> Vec<String> {
    declarations(source, &["attribute", "in"])
}

pub(crate) fn uniforms(source: &str) -> Vec<String> {
    declarations(source, &["uniform"])
}

fn declarations(source: &str, qualifiers: &[&str]) -> Vec<String> {
    let tokens: Vec<&str> = source
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|token| !token.is_empty())
        .collect();
    let mut names = Vec::new();
    for window in tokens.windows(3) {
        if qualifiers.contains(&window[0]) && is_vector(window[1]) {
            let name = window[2].to_string();
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

fn is_vector(value: &str) -> bool {
    matches!(value, "float" | "vec2" | "vec3" | "vec4")
}
