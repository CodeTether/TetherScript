use super::*;

pub(in crate::browser_js) fn fulfilled(value: JsValue) -> JsValue {
    object::from_state(state::PromiseState::Fulfilled(value))
}

pub(in crate::browser_js) fn resolved(value: JsValue) -> JsValue {
    object::from_state(state::settle(value))
}

pub(in crate::browser_js) fn rejected(reason: JsValue) -> JsValue {
    object::from_state(state::PromiseState::Rejected(reason))
}
