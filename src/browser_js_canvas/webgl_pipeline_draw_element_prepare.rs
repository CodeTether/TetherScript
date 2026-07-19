//! Indexed draw validation and immutable raster-work preparation.

use super::*;

pub(super) fn call(state: &mut WebGlState, args: &[JsValue]) -> Option<DrawCall> {
    let mode = validation::mode(state, args.first())?;
    let count = validation::non_negative(state, args.get(1))?;
    let (kind, width) = indices::kind(state, args.get(2))?;
    let offset = validation::non_negative(state, args.get(3))?;
    let element = indices::bound(state)?;
    if !indices::contains(&element.bytes, offset, count, width) {
        draw::invalid(state);
        return None;
    }
    let source = source::resolve(state)?;
    let call = build::call(state, source, count, |position| {
        indices::read(&element.bytes, offset, width, position)
    });
    if call.is_some() {
        state.push(format!("drawElements|{mode}|{count}|{kind}|{offset}"));
    }
    call
}
