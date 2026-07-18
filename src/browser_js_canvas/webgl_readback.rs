//! WebGL drawing-buffer byte extraction.

use super::super::super::surface::Surface;
use super::super::*;

pub(super) fn rgba(handle: &DomHandle, rect: [i64; 4]) -> Vec<u8> {
    super::super::super::store::with_surface(handle, |surface| from_surface(surface, rect))
}

fn from_surface(surface: &Surface, rect: [i64; 4]) -> Vec<u8> {
    let area = rect[2] as usize * rect[3] as usize;
    let mut bytes = Vec::with_capacity(area * 4);
    for row in 0..rect[3] {
        let source_y = (surface.height as i64 - 1).saturating_sub(rect[1].saturating_add(row));
        for column in 0..rect[2] {
            let source_x = rect[0].saturating_add(column);
            bytes.extend(super::super::super::pixels::at(surface, source_x, source_y));
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readback_emits_bottom_row_first() {
        let mut surface = Surface::new(1, 2);
        surface.pixels[0] = [1, 2, 3, 4];
        surface.pixels[1] = [5, 6, 7, 8];

        assert_eq!(
            from_surface(&surface, [0, 0, 1, 2]),
            vec![5, 6, 7, 8, 1, 2, 3, 4]
        );
    }
}
