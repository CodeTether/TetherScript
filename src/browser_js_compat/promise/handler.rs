use super::*;

pub(super) fn present(value: &JsValue) -> bool {
    !matches!(value, JsValue::Undefined | JsValue::Null)
}

pub(super) fn invoke(handler: JsValue, args: &[JsValue]) -> state::PromiseState {
    match js::call_function_with_this(handler, JsValue::Undefined, args) {
        Ok(value) => state::settle(value),
        Err(error) => state::PromiseState::Rejected(JsValue::String(error)),
    }
}
