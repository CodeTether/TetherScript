//! Z-index and paint-order resolution.
use super::types::{PositionType, PositionedElement};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintPhase {
    NegativePositioned,
    NormalFlow,
    AutoOrZeroPositioned,
    PositivePositioned,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaintRecord {
    pub index: usize,
    pub z_index: i32,
    pub phase: PaintPhase,
}

pub struct ZIndexResolver;

impl ZIndexResolver {
    pub fn paint_order<E>(elements: &[PositionedElement<E>]) -> Vec<PaintRecord> {
        let mut out = Vec::with_capacity(elements.len());
        for (index, el) in elements.iter().enumerate() {
            let positioned = el.position != PositionType::Static;
            let z = if positioned { el.effective_z_index() } else { 0 };
            let phase = if positioned && z < 0 {
                PaintPhase::NegativePositioned
            } else if !positioned {
                PaintPhase::NormalFlow
            } else if z == 0 {
                PaintPhase::AutoOrZeroPositioned
            } else {
                PaintPhase::PositivePositioned
            };
            out.push(PaintRecord { index, z_index: z, phase });
        }
        out.sort_by_key(|r| {
            let p = match r.phase {
                PaintPhase::NegativePositioned => 0,
                PaintPhase::NormalFlow => 1,
                PaintPhase::AutoOrZeroPositioned => 2,
                PaintPhase::PositivePositioned => 3,
            };
            (p, r.z_index, r.index)
        });
        out
    }
}
