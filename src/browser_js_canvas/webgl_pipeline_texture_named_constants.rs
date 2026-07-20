//! Texture constants installed on WebGL context objects.

use super::*;

const ALL: &[(&str, u32)] = &[
    ("TEXTURE_2D", constants::TEXTURE_2D),
    ("TEXTURE0", constants::TEXTURE0),
    ("TEXTURE1", constants::TEXTURE0 + 1),
    ("TEXTURE2", constants::TEXTURE0 + 2),
    ("TEXTURE3", constants::TEXTURE0 + 3),
    ("TEXTURE4", constants::TEXTURE0 + 4),
    ("TEXTURE5", constants::TEXTURE0 + 5),
    ("TEXTURE6", constants::TEXTURE0 + 6),
    ("TEXTURE7", constants::TEXTURE0 + 7),
    ("ACTIVE_TEXTURE", constants::ACTIVE_TEXTURE),
    ("TEXTURE_BINDING_2D", constants::TEXTURE_BINDING_2D),
    ("TEXTURE_MAG_FILTER", constants::TEXTURE_MAG_FILTER),
    ("TEXTURE_MIN_FILTER", constants::TEXTURE_MIN_FILTER),
    ("TEXTURE_WRAP_S", constants::TEXTURE_WRAP_S),
    ("TEXTURE_WRAP_T", constants::TEXTURE_WRAP_T),
    ("NEAREST", constants::NEAREST),
    ("LINEAR", constants::LINEAR),
    ("REPEAT", constants::REPEAT),
    ("CLAMP_TO_EDGE", constants::CLAMP_TO_EDGE),
    ("UNPACK_ALIGNMENT", constants::UNPACK_ALIGNMENT),
    ("UNPACK_FLIP_Y_WEBGL", constants::UNPACK_FLIP_Y_WEBGL),
    (
        "UNPACK_PREMULTIPLY_ALPHA_WEBGL",
        constants::UNPACK_PREMULTIPLY_ALPHA_WEBGL,
    ),
];

pub(super) fn install(obj: &mut HashMap<String, JsValue>) {
    for (name, value) in ALL {
        obj.insert((*name).into(), JsValue::Number(*value as f64));
    }
}
