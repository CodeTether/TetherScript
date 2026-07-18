//! WebGL `readPixels` API argument handling.

use super::super::*;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: DomHandle, version: u8) {
    obj.insert(
        "readPixels".into(),
        native("WebGLRenderingContext.readPixels", Some(7), move |args| {
            run(&handle, version, args)
        }),
    );
}

fn run(handle: &DomHandle, version: u8, args: &[JsValue]) -> Result<JsValue, String> {
    let read = super::args::parse(version, args)?;
    let bytes = super::readback::rgba(handle, read.rect);
    super::target::write(args.get(6), read.offset, &bytes)?;
    Ok(JsValue::Undefined)
}
