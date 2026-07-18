//! Source-mapped error-location serialization.

use crate::browser_agent::BrowserPage;
use crate::value::Value;

pub(super) fn stacks(page: &BrowserPage) -> Value {
    super::super::value::list(
        page.production_debug_report()
            .mapped_page_errors
            .into_iter()
            .map(|error| {
                super::super::super::super::value::map(vec![
                    (
                        "action",
                        super::super::super::super::value::string(error.action),
                    ),
                    (
                        "message",
                        super::super::super::super::value::string(error.message),
                    ),
                    (
                        "script_url",
                        super::super::super::super::value::string(error.generated.script_url),
                    ),
                    ("line", Value::Int(error.generated.line as i64)),
                    ("column", Value::Int(error.generated.column as i64)),
                    ("frame_count", Value::Int(error.stack.len() as i64)),
                ])
            })
            .collect(),
    )
}
