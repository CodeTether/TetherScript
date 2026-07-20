//! Constants for programmable WebGL resources and drawing.

pub(super) const TRIANGLES: u32 = 0x0004;
pub(super) const FLOAT: u32 = 0x1406;
pub(super) const VERTEX_SHADER: u32 = 0x8B31;
pub(super) const FRAGMENT_SHADER: u32 = 0x8B30;
pub(super) const DELETE_STATUS: u32 = 0x8B80;
pub(super) const COMPILE_STATUS: u32 = 0x8B81;
pub(super) const LINK_STATUS: u32 = 0x8B82;
pub(super) const ATTACHED_SHADERS: u32 = 0x8B85;
pub(super) const ACTIVE_UNIFORMS: u32 = 0x8B86;
pub(super) const ACTIVE_ATTRIBUTES: u32 = 0x8B89;
pub(super) const SHADER_TYPE: u32 = 0x8B4F;
pub(super) const CURRENT_PROGRAM: u32 = 0x8B8D;
pub(super) const ARRAY_BUFFER: u32 = 0x8892;
pub(super) const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
pub(super) const ARRAY_BUFFER_BINDING: u32 = 0x8894;
pub(super) const ELEMENT_ARRAY_BUFFER_BINDING: u32 = 0x8895;
pub(super) const BUFFER_SIZE: u32 = 0x8764;
pub(super) const BUFFER_USAGE: u32 = 0x8765;
pub(super) const UNSIGNED_SHORT: u32 = 0x1403;
pub(super) const UNSIGNED_INT: u32 = 0x1405;
pub(super) const STREAM_DRAW: u32 = 0x88E0;
pub(super) const STATIC_DRAW: u32 = 0x88E4;
pub(super) const DYNAMIC_DRAW: u32 = 0x88E8;
pub(super) const TEXTURE_2D: u32 = 0x0DE1;
pub(super) const TEXTURE0: u32 = 0x84C0;
pub(super) const ACTIVE_TEXTURE: u32 = 0x84E0;
pub(super) const TEXTURE_BINDING_2D: u32 = 0x8069;
pub(super) const TEXTURE_MAG_FILTER: u32 = 0x2800;
pub(super) const TEXTURE_MIN_FILTER: u32 = 0x2801;
pub(super) const TEXTURE_WRAP_S: u32 = 0x2802;
pub(super) const TEXTURE_WRAP_T: u32 = 0x2803;
pub(super) const NEAREST: u32 = 0x2600;
pub(super) const LINEAR: u32 = 0x2601;
pub(super) const REPEAT: u32 = 0x2901;
pub(super) const CLAMP_TO_EDGE: u32 = 0x812F;
pub(super) const UNPACK_ALIGNMENT: u32 = 0x0CF5;
pub(super) const UNPACK_FLIP_Y_WEBGL: u32 = 0x9240;
pub(super) const UNPACK_PREMULTIPLY_ALPHA_WEBGL: u32 = 0x9241;

pub(in crate::browser_js::canvas_host::webgl) const MAX_TEXTURE_UNITS: usize = 8;
pub(in crate::browser_js::canvas_host::webgl) const MAX_TEXTURE_SIZE: usize = 2048;

pub(super) fn usage(value: u32) -> bool {
    matches!(value, STREAM_DRAW | STATIC_DRAW | DYNAMIC_DRAW)
}
