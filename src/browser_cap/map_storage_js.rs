//! JavaScript snippets for storage-backed browserctl operations.

use super::super::call::BrowserCall;

pub(crate) fn eval(expression: String, scope: &'static str) -> BrowserCall {
    BrowserCall::new(
        "eval",
        scope,
        super::super::value::map_value(vec![
            ("action", super::super::value::str_value("eval")),
            ("expression", super::super::value::str_value(expression)),
        ]),
    )
}

pub(crate) fn js_string(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}
