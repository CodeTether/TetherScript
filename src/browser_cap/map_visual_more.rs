//! Additional visual action mappings.

use crate::value::Value;

use super::super::call::BrowserCall;

pub(crate) fn point_eval(args: &[Value]) -> Result<BrowserCall, String> {
    let x = super::super::args::expect_int("find_element_at", args, 0)?;
    let y = super::super::args::expect_int("find_element_at", args, 1)?;
    Ok(super::js::eval(
        format!("document.elementFromPoint({x},{y})?.outerHTML || null"),
        "browser.visual",
    ))
}

pub(crate) fn raw_visual(method: &str, args: &[Value]) -> Result<BrowserCall, String> {
    let mut entries = vec![
        ("action", super::super::value::str_value("visual_compare")),
        ("mode", super::super::value::str_value(method)),
        (
            "before",
            super::super::value::str_value(super::super::args::expect_str(method, args, 0)?),
        ),
    ];
    if method != "assert_screenshot_matches" {
        entries.push((
            "after",
            super::super::value::str_value(super::super::args::expect_str(method, args, 1)?),
        ));
    }
    Ok(BrowserCall::new(
        "visual_compare",
        "browser.visual",
        super::super::value::map_value(entries),
    ))
}
