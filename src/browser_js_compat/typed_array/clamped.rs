//! Shared `Uint8ClampedArray` construction and value coercion.

use super::*;

thread_local! {
    static PROTOTYPE: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    let prototype = super::prototype::object("Uint8ClampedArray", 1);
    PROTOTYPE.with(|slot| *slot.borrow_mut() = Some(prototype.clone()));
    window.insert(
        "Uint8ClampedArray".into(),
        super::constructor::uint8_clamped(prototype),
    );
}

pub(in crate::browser_js) fn from_bytes(bytes: Vec<u8>) -> Result<JsValue, String> {
    let source = JsValue::Array(Rc::new(RefCell::new(
        bytes
            .into_iter()
            .map(|byte| JsValue::Number(byte as f64))
            .collect(),
    )));
    js::call_function_with_this(constructor(), JsValue::Undefined, &[source])
}

pub(super) fn value(value: &JsValue) -> JsValue {
    JsValue::Number(byte(value) as f64)
}

pub(in crate::browser_js) fn byte(value: &JsValue) -> u8 {
    js::uint8_clamp(value)
}

fn constructor() -> JsValue {
    let prototype = PROTOTYPE
        .with(|slot| slot.borrow().clone())
        .unwrap_or_else(|| super::prototype::object("Uint8ClampedArray", 1));
    super::constructor::uint8_clamped(prototype)
}
