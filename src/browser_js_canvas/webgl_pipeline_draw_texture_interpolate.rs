//! Perspective-correct interpolation of two-component texture coordinates.

use super::*;

pub(super) fn coordinates(vertices: &[Vertex], weights: [f64; 3]) -> Option<[f64; 2]> {
    let mut numerator = [0.0; 2];
    let mut denominator = 0.0;
    for (index, vertex) in vertices.iter().enumerate() {
        let uv = vertex.1?;
        let w = vertex.0[3];
        if !w.is_finite() || w == 0.0 {
            return None;
        }
        let factor = weights[index] / w;
        numerator[0] += uv[0] * factor;
        numerator[1] += uv[1] * factor;
        denominator += factor;
    }
    (denominator != 0.0).then(|| [numerator[0] / denominator, numerator[1] / denominator])
}
