//! Coordinate hit resolution and validation.

use crate::browser::{find_layout_box_at_path, layout_document};
use crate::browser_agent::hit_style::bounds_for;
use crate::browser_agent::query::DomMatch;
use crate::browser_agent::resolve::Resolved;
use crate::browser_agent::{hit, BrowserPage};

pub(super) fn resolve(page: &BrowserPage, x: i64, y: i64) -> Result<(Resolved, i64, i64), String> {
    let page_x = x + page.session.scroll.x;
    let page_y = y + page.session.scroll.y;
    let target = hit::target_at(&page.session, page.viewport_width, page_x, page_y)
        .ok_or_else(|| format!("browser.mouse_click: no element at ({x}, {y})"))?;
    let element = super::dom::element_at_path(&page.session.document, &target.path)
        .cloned()
        .ok_or_else(|| format!("browser.mouse_click: invalid target at ({x}, {y})"))?;
    if element.attrs.contains_key("disabled") {
        return Err(format!(
            "browser.mouse_click: disabled {} at ({x}, {y})",
            target.label
        ));
    }
    let layout = layout_document(
        &page.session.document,
        &page.session.css,
        page.viewport_width,
    );
    let bounds = find_layout_box_at_path(&layout, &target.path)
        .map(bounds_for)
        .ok_or_else(|| format!("browser.mouse_click: target has no layout at ({x}, {y})"))?;
    Ok((
        Resolved {
            dom: DomMatch {
                path: target.path,
                element,
            },
            bounds,
        },
        page_x,
        page_y,
    ))
}
