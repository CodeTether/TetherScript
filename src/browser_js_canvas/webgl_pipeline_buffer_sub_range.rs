//! Bounds-checked writes into an existing buffer allocation.

use super::*;

pub(super) fn write(state: &mut WebGlState, target: u32, offset: i64, bytes: &[u8]) {
    if offset < 0 {
        invalid(state);
        return;
    }
    let start = offset as usize;
    let Some(bound) = validation::bound(state, target) else {
        return;
    };
    let Some(end) = start
        .checked_add(bytes.len())
        .filter(|end| *end <= bound.bytes.len())
    else {
        invalid(state);
        return;
    };
    bound.bytes[start..end].copy_from_slice(bytes);
}

fn invalid(state: &mut WebGlState) {
    webgl_error::record(state, webgl_constants::INVALID_VALUE);
}
