//! Coordinate click event dispatch and navigation defaults.

use crate::browser_agent::resolve::Resolved;
use crate::browser_agent::{downloads, BrowserPage};

pub(super) fn run(
    page: &mut BrowserPage,
    resolved: &Resolved,
    x: i64,
    y: i64,
    page_x: i64,
    page_y: i64,
) -> Result<(), String> {
    let is_download = downloads::is_anchor_download(resolved);
    let script = if is_download {
        downloads::click_script(&resolved.dom.path)
    } else {
        super::point_script::click(&resolved.dom.path, x, y, page_x, page_y)
    };
    let result = page.eval_js(&script)?;
    if is_download {
        downloads::record_anchor_download(page, resolved, &result);
        Ok(())
    } else {
        super::click::after_click(page, resolved, &result)
    }
}
