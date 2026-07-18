use crate::browser::{RasterImage, Rgba};

#[test]
fn png_contains_inflatable_rgba_scanlines() {
    let mut image = RasterImage::new(2, 1, Rgba::WHITE);
    image.pixels[0..4].copy_from_slice(&[1, 2, 3, 4]);
    let png = super::encode(&image);
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    let idat = chunk(&png, b"IDAT");
    let rows = crate::zlib::inflate_zlib(idat).unwrap();
    assert_eq!(&rows[0..5], &[0, 1, 2, 3, 4]);
    assert_eq!(rows.len(), 9);
}

fn chunk<'a>(png: &'a [u8], expected: &[u8; 4]) -> &'a [u8] {
    let mut offset = 8;
    loop {
        let length = u32::from_be_bytes(png[offset..offset + 4].try_into().unwrap()) as usize;
        let kind = &png[offset + 4..offset + 8];
        if kind == expected {
            return &png[offset + 8..offset + 8 + length];
        }
        offset += 12 + length;
    }
}
