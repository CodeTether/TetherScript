//! Mutating storage method mappings.

use crate::value::Value;

use super::super::call::BrowserCall;

pub(crate) fn set_cookie(args: &[Value]) -> Result<BrowserCall, String> {
    let cookie = super::super::args::expect_str("set_cookie", args, 0)?;
    Ok(super::js::eval(
        format!("document.cookie={};true", super::js::js_string(&cookie)),
        "browser.mutate.storage",
    ))
}

pub(crate) fn set_local(args: &[Value]) -> Result<BrowserCall, String> {
    let key = super::super::args::expect_str("set_local_storage", args, 0)?;
    let value = super::super::args::expect_str("set_local_storage", args, 1)?;
    Ok(super::js::eval(
        format!(
            "localStorage.setItem({},{});true",
            super::js::js_string(&key),
            super::js::js_string(&value)
        ),
        "browser.mutate.storage",
    ))
}

pub(crate) fn clear(args: &[Value]) -> Result<BrowserCall, String> {
    super::super::args::no_args("clear_storage", args)?;
    Ok(BrowserCall::new(
        "clear_storage",
        "browser.mutate.storage",
        super::super::value::map_value(vec![(
            "action",
            super::super::value::str_value("clear_storage"),
        )]),
    ))
}
