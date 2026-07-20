use super::case::{assert_case, Case};

#[path = "canvas_webgl_2d.rs"]
mod canvas_2d;
#[path = "canvas_webgl_draw.rs"]
mod draw;
#[path = "canvas_webgl_indexed.rs"]
mod indexed;
#[path = "canvas_webgl_state.rs"]
mod state;
#[path = "canvas_webgl_texture.rs"]
mod texture;

const CASE: Case = Case {
    area: "html/canvas/offscreen/webgl",
    wpt_shape: "2D ImageData and array/index/texture-backed WebGL triangles update native raster pixels",
    unsupported: &[
        "WebGL HTML media texture sources, mipmaps, cube/3D textures, depth/stencil, blending, and general GLSL ES",
    ],
};

pub fn run() {
    assert_case(&CASE);
    canvas_2d::run();
    state::run();
    draw::run();
    indexed::run();
    texture::run();
}
