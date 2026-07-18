//! Native viewport configuration mapping.

use crate::value::Value;

use super::call::BrowserCall;

pub(crate) fn prepare(args: &[Value]) -> Result<BrowserCall, String> {
    let entries = vec![
        (
            "width",
            Value::Int(super::args::expect_int("set_viewport", args, 0)?),
        ),
        (
            "height",
            Value::Int(super::args::expect_int("set_viewport", args, 1)?),
        ),
    ];
    Ok(BrowserCall::new(
        "set_viewport",
        super::actions::scope_for_action("set_viewport").unwrap(),
        super::value::map_value(
            std::iter::once(("action", super::value::str_value("set_viewport")))
                .chain(entries)
                .collect(),
        ),
    ))
}
