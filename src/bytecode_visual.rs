//! Human-readable bytecode visualization.

mod chunk;
mod instr;

/// Render a compiled chunk as annotated text for teaching and inspection.
pub(crate) fn render(chunk: &crate::bytecode::Chunk) -> String {
    chunk::render(chunk)
}
