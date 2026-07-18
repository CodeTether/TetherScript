//! Consecutive-layout stability checks for user-like actions.

use crate::browser_agent::locator::Locator;
use crate::browser_agent::page::BrowserPage;
use crate::browser_agent::resolve::Resolved;

pub(crate) fn run<T>(
    page: &mut BrowserPage,
    action: &str,
    locator: &Locator,
    mut attempt: impl FnMut(&mut BrowserPage) -> Result<(Resolved, T), String>,
) -> Result<(Resolved, T), String> {
    let mut previous = None;
    super::run(page, action, locator, |page| {
        let value = match attempt(page) {
            Ok(value) => value,
            Err(error) => {
                previous = None;
                return Err(error);
            }
        };
        let current = value.0.bounds;
        match previous.replace(current) {
            Some(last) if last == current => Ok(value),
            Some(last) => Err(format!(
                "locator {:?} failed actionability check stable: bounds changed from {:?} to {:?}",
                locator.kind, last, current
            )),
            None => Err(format!(
                "locator {:?} awaiting second stable layout observation at {:?}",
                locator.kind, current
            )),
        }
    })
}
