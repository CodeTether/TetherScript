//! RGBA-to-minifb framebuffer conversion.

use crate::browser::RasterImage;

pub(super) fn convert(image: &RasterImage) -> Result<Vec<u32>, String> {
    let expected = image.width.saturating_mul(image.height).saturating_mul(4);
    if image.pixels.len() != expected {
        return Err(format!(
            "render_surface.present: expected {expected} RGBA bytes, got {}",
            image.pixels.len()
        ));
    }
    Ok(image.pixels.chunks_exact(4).map(rgb).collect())
}

fn rgb(rgba: &[u8]) -> u32 {
    ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::Rgba;

    #[test]
    fn converts_rgba_to_minifb_rgb() {
        let image = RasterImage::new(
            1,
            1,
            Rgba {
                r: 0x12,
                g: 0x34,
                b: 0x56,
                a: 0x78,
            },
        );
        assert_eq!(convert(&image).unwrap(), vec![0x12_34_56]);
    }
}
