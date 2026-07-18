//! `putImageData` writes into the native canvas surface.

use super::*;

pub(super) fn put(handle: &DomHandle, args: &[JsValue]) -> Result<JsValue, String> {
    let source = super::source::parse(args.first())?;
    let destination_x = super::geometry::i64_value(args.get(1));
    let destination_y = super::geometry::i64_value(args.get(2));
    let region = super::dirty::region(args, source.width, source.height);
    super::store::mutate(handle, |surface| {
        write(surface, &source, &region, destination_x, destination_y);
        surface.commands.push(format!(
            "putImageData|{destination_x}|{destination_y}|{}|{}",
            region.x1.saturating_sub(region.x0),
            region.y1.saturating_sub(region.y0)
        ));
    });
    Ok(JsValue::Undefined)
}

fn write(
    surface: &mut super::surface::Surface,
    source: &super::source::Source,
    region: &super::dirty::Region,
    destination_x: i64,
    destination_y: i64,
) {
    for source_y in region.y0..region.y1 {
        for source_x in region.x0..region.x1 {
            let x = destination_x.saturating_add(source_x as i64);
            let y = destination_y.saturating_add(source_y as i64);
            if x < 0 || y < 0 || x >= surface.width as i64 || y >= surface.height as i64 {
                continue;
            }
            let source_index = source_y * source.width + source_x;
            let destination_index = y as usize * surface.width as usize + x as usize;
            if let Some(pixel) = surface.pixels.get_mut(destination_index) {
                *pixel = source.pixels[source_index];
            }
        }
    }
}
