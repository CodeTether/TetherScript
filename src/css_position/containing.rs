//! Containing block resolution.

use super::types::{PositionType, PositionedElement, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainingBlock {
    pub rect: Rect,
}

impl ContainingBlock {
    pub fn viewport(width: f32, height: f32) -> Self {
        Self { rect: Rect { x: 0.0, y: 0.0, width, height } }
    }
}

pub struct ContainingBlockResolver;

impl ContainingBlockResolver {
    pub fn resolve<E>(
        elements: &[PositionedElement<E>],
        index: usize,
        viewport: ContainingBlock,
    ) -> ContainingBlock {
        let el = &elements[index];
        match el.position {
            PositionType::Fixed => viewport,
            PositionType::Absolute => {
                let mut p = el.parent;
                while let Some(i) = p {
                    if elements[i].is_positioned() {
                        return Self::content_box(&elements[i]);
                    }
                    p = elements[i].parent;
                }
                viewport
            }
            PositionType::Static | PositionType::Relative | PositionType::Sticky => {
                el.parent.map(|i| Self::content_box(&elements[i])).unwrap_or(viewport)
            }
        }
    }

    pub fn content_box<E>(el: &PositionedElement<E>) -> ContainingBlock {
        let x = el.computed_x + el.border.left + el.padding.left;
        let y = el.computed_y + el.border.top + el.padding.top;
        let w = el.width - el.border.left - el.border.right - el.padding.left - el.padding.right;
        let h = el.height - el.border.top - el.border.bottom - el.padding.top - el.padding.bottom;
        ContainingBlock { rect: Rect { x, y, width: w.max(0.0), height: h.max(0.0) } }
    }
}
