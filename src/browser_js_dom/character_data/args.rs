use super::*;

pub(super) fn text(args: &[JsValue], index: usize) -> String {
    args.get(index).unwrap_or(&JsValue::Undefined).display()
}

pub(super) fn unsigned(args: &[JsValue], index: usize) -> usize {
    let value = args.get(index).unwrap_or(&JsValue::Undefined);
    let number = match value {
        JsValue::Number(value) => *value,
        JsValue::Null => 0.0,
        JsValue::Bool(value) => u8::from(*value) as f64,
        JsValue::String(value) => value.trim().parse().unwrap_or(f64::NAN),
        _ => f64::NAN,
    };
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let modulo = 4_294_967_296.0;
    ((number.trunc() % modulo + modulo) % modulo) as usize
}
