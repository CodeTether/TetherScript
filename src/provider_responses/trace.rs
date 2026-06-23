//! Opt-in Responses event tracing.

use crate::value::Value;

pub(super) fn event(value: &Value) {
    if std::env::var_os("TETHERSCRIPT_PROVIDER_RESPONSES_TRACE").is_none() {
        return;
    }
    match crate::json::encode_to_string(value) {
        Ok(text) => eprintln!("provider.responses.event {text}"),
        Err(error) => eprintln!("provider.responses.event <encode error: {error}>"),
    }
}
