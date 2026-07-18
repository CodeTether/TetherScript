use super::case::{assert_case, Case};

#[path = "canvas_webgl_2d.rs"]
mod canvas_2d;
#[path = "canvas_webgl_draw.rs"]
mod draw;
#[path = "canvas_webgl_state.rs"]
mod state;

const CASE: Case = Case {
    area: "html/canvas/offscreen/webgl",
    wpt_shape: "2D ImageData and buffer-backed WebGL triangles update native raster pixels",
    unsupported: &["WebGL textures, indexed drawing, depth/stencil, blending, and general GLSL ES"],
};

pub fn run() {
    assert_case(&CASE);
    canvas_2d::run();
    state::run();
    draw::run();
}
