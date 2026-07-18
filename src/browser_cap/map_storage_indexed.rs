//! Native IndexedDB summary action mapping.

use crate::value::Value;

use super::super::call::BrowserCall;

pub(super) fn prepare(args: &[Value]) -> Result<BrowserCall, String> {
    super::super::args::no_args("indexed_db_summary", args)?;
    Ok(BrowserCall::new(
        "indexed_db_summary",
        "browser.inspect.storage",
        super::super::value::map_value(vec![(
            "action",
            super::super::value::str_value("indexed_db_summary"),
        )]),
    ))
}
