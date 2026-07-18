use super::*;

pub(super) fn run(handle: &DomHandle, options: Option<&JsValue>) -> Result<(), String> {
    let (block, inline) = alignment::options(options);
    reveal::ancestors(handle, block, inline)?;
    let rect = rect::visible(handle);
    let viewport = window_host::scroll::metrics();
    let x = alignment::axis(rect.0, rect.2, viewport.x, viewport.width, inline).max(0);
    let y = alignment::axis(rect.1, rect.3, viewport.y, viewport.height, block).max(0);
    window_host::scroll::to(x, y).map(|_| ())
}
