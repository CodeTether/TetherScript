//! Clip-space to viewport-space vertex projection and edge tests.

use super::*;

pub(super) fn screen_triangle(vertices: &[Vertex], viewport: [i64; 4]) -> Option<[[f64; 2]; 3]> {
    let mut points = [[0.0; 2]; 3];
    for (index, vertex) in vertices.iter().enumerate() {
        let [x, y, _, w] = vertex.0;
        if !w.is_finite() || w == 0.0 {
            return None;
        }
        points[index] = [
            viewport[0] as f64 + (x / w + 1.0) * viewport[2] as f64 / 2.0,
            viewport[1] as f64 + (y / w + 1.0) * viewport[3] as f64 / 2.0,
        ];
    }
    Some(points)
}

pub(super) fn edge(a: [f64; 2], b: [f64; 2], point: [f64; 2]) -> f64 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}

pub(super) fn weights(edges: [f64; 3], area: f64) -> [f64; 3] {
    [edges[1] / area, edges[2] / area, edges[0] / area]
}
