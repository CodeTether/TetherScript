//! Canvas surface metadata synchronization.

use super::*;

pub(super) fn summary(handle: &DomHandle) -> String {
    super::store::with_surface(handle, |surface| {
        format!(
            "{}x{}:{}:{}",
            surface.width,
            surface.height,
            surface.commands.len(),
            super::pixels::checksum(surface)
        )
    })
}

pub(super) fn sync_attrs(handle: &DomHandle, surface: &super::surface::Surface) {
    handle.with_node_mut(|node| {
        if let Node::Element(el) = node {
            el.attrs
                .insert("data-agent-canvas-width".into(), surface.width.to_string());
            el.attrs.insert(
                "data-agent-canvas-height".into(),
                surface.height.to_string(),
            );
            el.attrs.insert(
                "data-agent-canvas-commands".into(),
                surface.commands.join(";"),
            );
            el.attrs.insert(
                "data-agent-canvas-checksum".into(),
                super::pixels::checksum(surface).to_string(),
            );
        }
    });
}
