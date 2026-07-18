//! Scroll target calculation for native host actions.

use crate::browser_agent::{resolve, scroll, BrowserPage, Locator};
use crate::browser_session::ScrollState;

pub(super) fn resolve(
    page: &BrowserPage,
    selector: Option<&str>,
    coordinates: Option<(i64, i64)>,
) -> Result<ScrollState, String> {
    if selector.is_none() && coordinates.is_none() {
        return Err("browser.scroll: expected selector or x/y coordinates".into());
    }
    let mut target = page.session.scroll.clone();
    if let Some(selector) = selector {
        let resolved =
            resolve::resolve(&page.session, page.viewport_width, &Locator::css(selector))?;
        scroll::into_view(
            &mut target,
            resolved.bounds,
            page.viewport_width,
            page.viewport_height,
        );
    }
    if let Some((x, y)) = coordinates {
        let base = if selector.is_some() {
            target
        } else {
            ScrollState { x: 0, y: 0 }
        };
        target = ScrollState {
            x: (base.x + x).max(0),
            y: (base.y + y).max(0),
        };
    }
    Ok(target)
}
