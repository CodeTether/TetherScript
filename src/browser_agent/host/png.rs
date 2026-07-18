//! Dependency-free RGBA PNG encoding for native browser screenshots.

use crate::browser::RasterImage;

#[cfg(test)]
#[path = "png_tests.rs"]
mod tests;

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

pub(super) fn encode(image: &RasterImage) -> Vec<u8> {
    let mut output = SIGNATURE.to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&(image.width as u32).to_be_bytes());
    header.extend_from_slice(&(image.height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    super::png_chunk::push(&mut output, b"IHDR", &header);
    let compressed = super::png_zlib::stored(&scanlines(image));
    super::png_chunk::push(&mut output, b"IDAT", &compressed);
    super::png_chunk::push(&mut output, b"IEND", &[]);
    output
}

fn scanlines(image: &RasterImage) -> Vec<u8> {
    let stride = image.width * 4;
    let mut rows = Vec::with_capacity(image.height * (stride + 1));
    for row in image.pixels.chunks(stride) {
        rows.push(0);
        rows.extend_from_slice(row);
    }
    rows
}
