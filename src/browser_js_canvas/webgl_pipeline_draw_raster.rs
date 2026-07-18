//! Pixel-center triangle rasterization into the canvas RGBA surface.

use super::super::super::super::surface::Surface;
use super::*;

pub(super) fn draw(surface: &mut Surface, call: &DrawCall) {
    for triangle in call.vertices.chunks_exact(3) {
        let Some(points) = geometry::screen_triangle(triangle, call.viewport) else {
            continue;
        };
        raster_triangle(surface, points, call);
    }
}

fn raster_triangle(surface: &mut Surface, points: [[f64; 2]; 3], call: &DrawCall) {
    let area = geometry::edge(points[0], points[1], points[2]);
    if area == 0.0 || surface.pixels.is_empty() {
        return;
    }
    for y in 0..surface.height as usize {
        for x in 0..surface.width as usize {
            if !pixels::inside_scissor(x, y, call.scissor) {
                continue;
            }
            let point = [x as f64 + 0.5, y as f64 + 0.5];
            let edges = [
                geometry::edge(points[0], points[1], point),
                geometry::edge(points[1], points[2], point),
                geometry::edge(points[2], points[0], point),
            ];
            if edges.iter().all(|value| *value >= 0.0) || edges.iter().all(|value| *value <= 0.0) {
                pixels::write(surface, x, y, call)
            }
        }
    }
}
