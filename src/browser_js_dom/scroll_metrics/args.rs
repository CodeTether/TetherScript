use super::*;

pub(super) fn absolute(args: &[JsValue], current: state::Position) -> state::Position {
    let Some(first) = args.first() else {
        return state::Position::default();
    };
    if let JsValue::Object(object) = first {
        let object = object.borrow();
        return state::Position {
            left: property(&object, "left").unwrap_or(current.left),
            top: property(&object, "top").unwrap_or(current.top),
        };
    }
    state::Position {
        left: number(Some(first)),
        top: number(args.get(1)),
    }
}

pub(super) fn relative(args: &[JsValue]) -> state::Position {
    let Some(first) = args.first() else {
        return state::Position::default();
    };
    if let JsValue::Object(object) = first {
        let object = object.borrow();
        return state::Position {
            left: property(&object, "left").unwrap_or(0),
            top: property(&object, "top").unwrap_or(0),
        };
    }
    state::Position {
        left: number(Some(first)),
        top: number(args.get(1)),
    }
}

pub(super) fn number(value: Option<&JsValue>) -> i64 {
    let number = match value {
        Some(JsValue::Number(number)) => *number,
        Some(JsValue::Bool(true)) => 1.0,
        Some(JsValue::Bool(false) | JsValue::Null | JsValue::Undefined) | None => 0.0,
        Some(value) => value.display().trim().parse().unwrap_or(0.0),
    };
    if number.is_finite() {
        number.trunc() as i64
    } else {
        0
    }
}

fn property(object: &HashMap<String, JsValue>, name: &str) -> Option<i64> {
    object.get(name).map(|value| number(Some(value)))
}
