//! DOM attribute and annotated React metadata views.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

pub(super) fn attributes(page: &BrowserPage, selector: &str) -> Result<Value, String> {
    let element = super::find(page, selector)?;
    Ok(super::super::value::list(
        element
            .attrs
            .iter()
            .map(|(name, value)| {
                super::super::super::super::value::map(vec![
                    ("name", super::super::super::super::value::string(name)),
                    ("value", super::super::super::super::value::string(value)),
                ])
            })
            .collect(),
    ))
}

pub(super) fn metadata(page: &BrowserPage, selector: &str, name: &str) -> Result<Value, String> {
    Ok(super::super::value::optional(
        super::find(page, selector)?.attrs.get(name).cloned(),
    ))
}
