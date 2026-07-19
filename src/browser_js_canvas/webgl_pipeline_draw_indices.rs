//! Element-array binding, range validation, and index decoding.

use super::*;

pub(super) fn kind(state: &mut WebGlState, value: Option<&JsValue>) -> Option<(u32, usize)> {
    let kind = webgl_values::u32_value(value);
    let width = match kind {
        webgl_constants::UNSIGNED_BYTE => 1,
        constants::UNSIGNED_SHORT => 2,
        constants::UNSIGNED_INT if state.version >= 2 => 4,
        _ => {
            webgl_error::record(state, webgl_constants::INVALID_ENUM);
            return None;
        }
    };
    Some((kind, width))
}

pub(super) fn bound(state: &mut WebGlState) -> Option<buffer_state::Buffer> {
    let id = state.pipeline.bound_element_array_buffer;
    let Some(buffer) = id
        .and_then(|id| state.pipeline.buffers.get(&id))
        .filter(|buffer| !buffer.deleted)
        .cloned()
    else {
        draw::invalid(state);
        return None;
    };
    Some(buffer)
}

pub(super) fn contains(bytes: &[u8], offset: usize, count: usize, width: usize) -> bool {
    offset.checked_rem(width) == Some(0)
        && count
            .checked_mul(width)
            .and_then(|length| offset.checked_add(length))
            .is_some_and(|end| end <= bytes.len())
}

pub(super) fn read(bytes: &[u8], offset: usize, width: usize, position: usize) -> Option<usize> {
    let start = offset.checked_add(position.checked_mul(width)?)?;
    match width {
        1 => Some(*bytes.get(start)? as usize),
        2 => Some(u16::from_le_bytes(bytes.get(start..start + 2)?.try_into().ok()?) as usize),
        4 => Some(u32::from_le_bytes(bytes.get(start..start + 4)?.try_into().ok()?) as usize),
        _ => None,
    }
}
