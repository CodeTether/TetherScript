//! Programmable-pipeline constants installed on WebGL contexts.

use super::*;

const ALL: &[(&str, u32)] = &[
    ("TRIANGLES", constants::TRIANGLES),
    ("FLOAT", constants::FLOAT),
    ("VERTEX_SHADER", constants::VERTEX_SHADER),
    ("FRAGMENT_SHADER", constants::FRAGMENT_SHADER),
    ("DELETE_STATUS", constants::DELETE_STATUS),
    ("COMPILE_STATUS", constants::COMPILE_STATUS),
    ("LINK_STATUS", constants::LINK_STATUS),
    ("ATTACHED_SHADERS", constants::ATTACHED_SHADERS),
    ("ACTIVE_UNIFORMS", constants::ACTIVE_UNIFORMS),
    ("ACTIVE_ATTRIBUTES", constants::ACTIVE_ATTRIBUTES),
    ("SHADER_TYPE", constants::SHADER_TYPE),
    ("CURRENT_PROGRAM", constants::CURRENT_PROGRAM),
    ("ARRAY_BUFFER", constants::ARRAY_BUFFER),
    ("ELEMENT_ARRAY_BUFFER", constants::ELEMENT_ARRAY_BUFFER),
    ("ARRAY_BUFFER_BINDING", constants::ARRAY_BUFFER_BINDING),
    (
        "ELEMENT_ARRAY_BUFFER_BINDING",
        constants::ELEMENT_ARRAY_BUFFER_BINDING,
    ),
    ("BUFFER_SIZE", constants::BUFFER_SIZE),
    ("BUFFER_USAGE", constants::BUFFER_USAGE),
    ("UNSIGNED_SHORT", constants::UNSIGNED_SHORT),
    ("UNSIGNED_INT", constants::UNSIGNED_INT),
    ("STREAM_DRAW", constants::STREAM_DRAW),
    ("STATIC_DRAW", constants::STATIC_DRAW),
    ("DYNAMIC_DRAW", constants::DYNAMIC_DRAW),
];

pub(super) fn install(obj: &mut HashMap<String, JsValue>) {
    for (name, value) in ALL {
        obj.insert((*name).into(), JsValue::Number(*value as f64));
    }
}
