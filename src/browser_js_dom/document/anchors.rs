use super::super::*;
use super::live_query;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    obj.insert("anchors".into(), collection(handle));
}

fn collection(handle: &DomHandle) -> JsValue {
    live_query::collection(handle, |el| el.tag == "a" && el.attrs.contains_key("name"))
}
