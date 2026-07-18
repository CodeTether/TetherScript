//! Polling network wait action mappings.

use crate::value::Value;

use super::super::call::BrowserCall;

pub(crate) fn prepare(method: &str, args: &[Value]) -> Result<BrowserCall, String> {
    let kind = method.strip_prefix("wait_for_").unwrap_or(method);
    Ok(super::call(
        "network_log",
        vec![
            (
                "url_contains",
                super::super::value::str_value(super::super::args::expect_str(method, args, 0)?),
            ),
            ("limit", Value::Int(1)),
            ("wait_kind", super::super::value::str_value(kind)),
            (
                "timeout_ms",
                Value::Int(super::super::args::optional_int(args, 1, 30_000)?),
            ),
        ],
    ))
}
