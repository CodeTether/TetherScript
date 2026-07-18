use super::super::*;

const WIDTH: &str = "__tsOrientationWidth";
const HEIGHT: &str = "__tsOrientationHeight";

pub(super) fn initialize(object: &Rc<RefCell<HashMap<String, JsValue>>>) {
    let mut object = object.borrow_mut();
    object.insert(
        WIDTH.into(),
        JsValue::Number(constants::DEFAULT_VIEWPORT_WIDTH as f64),
    );
    object.insert(
        HEIGHT.into(),
        JsValue::Number(constants::DEFAULT_VIEWPORT_HEIGHT as f64),
    );
}

pub(super) fn remember(object: &Rc<RefCell<HashMap<String, JsValue>>>, width: f64, height: f64) {
    let mut object = object.borrow_mut();
    object.insert(WIDTH.into(), JsValue::Number(width));
    object.insert(HEIGHT.into(), JsValue::Number(height));
}

pub(super) fn current(object: &Rc<RefCell<HashMap<String, JsValue>>>) -> super::value::Snapshot {
    let object = object.borrow();
    super::value::viewport(number(object.get(WIDTH)), number(object.get(HEIGHT)))
}

fn number(value: Option<&JsValue>) -> f64 {
    match value {
        Some(JsValue::Number(value)) => *value,
        _ => 0.0,
    }
}
