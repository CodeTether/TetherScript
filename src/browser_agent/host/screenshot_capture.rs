//! Viewport or selector-scoped screenshot capture.

use crate::browser::RasterImage;
use crate::browser_agent::Locator;
use crate::value::Value;

use super::super::state::HostState;

pub(super) fn image(state: &HostState, payload: &Value) -> Result<RasterImage, String> {
    let Some(selector) = super::super::value::optional_string(payload, "selector")? else {
        return state.page.screenshot();
    };
    state
        .page
        .element_screenshot(&Locator::css(selector))
        .map(|shot| shot.image)
}
