use super::*;

pub(super) fn ancestors(
    handle: &DomHandle,
    block: alignment::Alignment,
    inline: alignment::Alignment,
) -> Result<(), String> {
    for depth in (1..handle.path.len()).rev() {
        let ancestor = DomHandle {
            root: handle.root.clone(),
            path: handle.path[..depth].to_vec(),
        };
        reveal_in(handle, &ancestor, block, inline)?;
    }
    Ok(())
}

fn reveal_in(
    target: &DomHandle,
    ancestor: &DomHandle,
    block: alignment::Alignment,
    inline: alignment::Alignment,
) -> Result<(), String> {
    let geometry = geometry::measure(ancestor);
    if geometry.max_left() == 0 && geometry.max_top() == 0 {
        return Ok(());
    }
    let target = rect::visible(target);
    let parent = rect::visible(ancestor);
    let current = apply::current(ancestor);
    let left = reveal_alignment::aligned(
        current.left,
        target.0,
        target.2,
        parent.0 + geometry.client_left,
        geometry.client_width,
        inline,
    );
    let top = reveal_alignment::aligned(
        current.top,
        target.1,
        target.3,
        parent.1 + geometry.client_top,
        geometry.client_height,
        block,
    );
    apply::to(ancestor, state::Position { left, top }).map(|_| ())
}
