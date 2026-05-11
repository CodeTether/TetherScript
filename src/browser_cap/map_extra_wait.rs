//! Wait-related argument mappings for browserctl.

use crate::value::Value;

use crate::browser_cap::call::BrowserCall;

pub(crate) fn selector(args: &[Value]) -> Result<BrowserCall, String> {
    crate::browser_cap::args::expect_str("wait_for_selector", args, 0)?;
    Ok(call(
        "wait_for_selector",
        vec![("selector", args[0].clone())],
    ))
}

pub(crate) fn text(args: &[Value]) -> Result<BrowserCall, String> {
    crate::browser_cap::args::expect_str("wait_for_text", args, 0)?;
    Ok(call("wait_for_text", vec![("text", args[0].clone())]))
}

pub(crate) fn idle(args: &[Value]) -> Result<BrowserCall, String> {
    crate::browser_cap::args::no_args("wait_for_network_idle", args)?;
    Ok(call("wait_for_network_idle", Vec::new()))
}

fn call(action: &str, mut entries: Vec<(&str, Value)>) -> BrowserCall {
    entries.insert(0, ("action", crate::browser_cap::value::str_value(action)));
    BrowserCall::new(
        action,
        crate::browser_cap::actions::scope_for_action(action).unwrap(),
        crate::browser_cap::value::map_value(entries),
    )
}
