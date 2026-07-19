//! Draw-call validation and immutable raster work assembly.

use super::*;

pub(super) fn call(state: &mut WebGlState, args: &[JsValue]) -> Option<DrawCall> {
    let mode = validation::mode(state, args.first())?;
    let first = validation::non_negative(state, args.get(1))?;
    let count = validation::non_negative(state, args.get(2))?;
    let source = source::resolve(state)?;
    let call = build::call(state, source, count, |position| {
        first.checked_add(position)
    });
    if call.is_some() {
        state.push(format!("drawArrays|{mode}|{first}|{count}"));
    }
    call
}
