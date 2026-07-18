//! Selection of the linked position attribute.

use super::*;

pub(super) fn location(program: &shader_state::Program) -> Option<u32> {
    program
        .attributes
        .iter()
        .find(|(name, _)| name.to_ascii_lowercase().contains("position"))
        .map(|(_, location)| *location)
        .or_else(|| program.attributes.values().copied().min())
}
