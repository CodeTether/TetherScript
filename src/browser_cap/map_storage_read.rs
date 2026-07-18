//! Native storage observation action mappings.

use crate::value::Value;

use super::super::call::BrowserCall;

pub(super) fn prepare(method: &str, args: &[Value]) -> Result<BrowserCall, String> {
    super::super::args::no_args(method, args)?;
    Ok(BrowserCall::new(
        method,
        "browser.inspect.storage",
        super::super::value::map_value(vec![("action", super::super::value::str_value(method))]),
    ))
}
