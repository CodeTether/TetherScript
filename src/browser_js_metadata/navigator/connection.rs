use super::*;
use std::cell::RefCell;
use std::rc::Rc;

pub(super) fn install(navigator: &mut HashMap<String, JsValue>) {
    let connection = navigator
        .get("connection")
        .cloned()
        .unwrap_or_else(network_information);
    enrich_network_information(&connection);
    navigator.insert("connection".into(), connection.clone());
    navigator.insert("networkInformation".into(), connection);
}

fn network_information() -> JsValue {
    let mut info = HashMap::new();
    info.insert("onchange".into(), JsValue::Null);
    let value = JsValue::Object(Rc::new(RefCell::new(info)));
    enrich_network_information(&value);
    value
}

fn enrich_network_information(value: &JsValue) {
    let JsValue::Object(object) = value else {
        return;
    };
    let mut info = object.borrow_mut();
    info.entry("effectiveType".into())
        .or_insert_with(|| JsValue::String("4g".into()));
    info.entry("type".into())
        .or_insert_with(|| JsValue::String("wifi".into()));
    info.entry("downlink".into())
        .or_insert(JsValue::Number(10.0));
    info.entry("downlinkMax".into())
        .or_insert(JsValue::Number(10.0));
    info.entry("rtt".into()).or_insert(JsValue::Number(50.0));
    info.entry("saveData".into())
        .or_insert(JsValue::Bool(false));
}
