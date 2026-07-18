//! Floating-point vertex decoding from raw `ARRAY_BUFFER` bytes.

use super::*;

pub(super) fn read(
    bytes: &[u8],
    attribute: &buffer_state::Attribute,
    index: usize,
) -> Option<Vertex> {
    let start = attribute
        .offset
        .checked_add(index.checked_mul(attribute.stride)?)?;
    let mut values = [0.0, 0.0, 0.0, 1.0];
    for (component, value) in values.iter_mut().enumerate().take(attribute.size) {
        let offset = start.checked_add(component * 4)?;
        let raw: [u8; 4] = bytes.get(offset..offset + 4)?.try_into().ok()?;
        *value = f32::from_le_bytes(raw) as f64;
    }
    Some(Vertex(values))
}
