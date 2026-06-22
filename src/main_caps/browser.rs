//! Browser capability construction for CLI grants.

use std::rc::Rc;

use crate::browser_cap;
use crate::capability::Authority;

pub(super) fn authority(
    endpoint: &Option<String>,
    origins: &[String],
    scopes: &[String],
) -> Option<Rc<dyn Authority>> {
    endpoint.as_ref().map(|endpoint| {
        browser_cap::BrowserAuthority::new(endpoint, origins.to_vec(), scope_list(scopes))
    })
}

fn scope_list(scopes: &[String]) -> Vec<String> {
    if !scopes.is_empty() {
        return scopes.to_vec();
    }
    vec![
        "browser.navigate".into(),
        "browser.interact".into(),
        "browser.inspect.dom".into(),
        "browser.inspect.console".into(),
        "browser.inspect.network".into(),
        "browser.screenshot".into(),
    ]
}
