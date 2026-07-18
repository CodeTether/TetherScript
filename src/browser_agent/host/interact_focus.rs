//! Focus tracking after locator-backed host interactions.

use crate::browser_agent::Locator;
use crate::value::Value;

pub(super) fn locator(action: &str, payload: &Value) -> Result<Option<Locator>, String> {
    match action {
        "click" | "fill" | "fill_native" | "type" => Ok(Some(Locator::css(
            super::value::string_field(payload, "selector")?,
        ))),
        "click_text" => Ok(Some(Locator::text_exact(super::value::string_field(
            payload, "text",
        )?))),
        _ => Ok(None),
    }
}
