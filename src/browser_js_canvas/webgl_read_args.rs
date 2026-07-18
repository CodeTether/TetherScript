//! WebGL `readPixels` argument decoding and validation.

use super::super::*;

pub(super) struct ReadArgs {
    pub rect: [i64; 4],
    pub offset: usize,
}

pub(super) fn parse(version: u8, args: &[JsValue]) -> Result<ReadArgs, String> {
    let rect = rect(args);
    super::area::validate(rect)?;
    super::format::validate(args.get(4), args.get(5))?;
    let offset = if version >= 2 {
        offset(args.get(7))?
    } else {
        0
    };
    Ok(ReadArgs { rect, offset })
}

fn rect(args: &[JsValue]) -> [i64; 4] {
    [
        super::super::webgl_values::i64_value(args.first()),
        super::super::webgl_values::i64_value(args.get(1)),
        super::super::webgl_values::i64_value(args.get(2)),
        super::super::webgl_values::i64_value(args.get(3)),
    ]
}

fn offset(value: Option<&JsValue>) -> Result<usize, String> {
    let value = super::super::webgl_values::i64_value(value);
    usize::try_from(value)
        .map_err(|_| "WebGLRenderingContext.readPixels: negative destination offset".into())
}
