use super::super::*;
use super::live_query;

pub(super) fn install(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    obj.insert("readyState".into(), JsValue::String("complete".into()));
    obj.insert("forms".into(), tagged_collection(handle, "form"));
    obj.insert("images".into(), tagged_collection(handle, "img"));
    obj.insert("scripts".into(), tagged_collection(handle, "script"));
    obj.insert("links".into(), link_collection(handle));
}

fn tagged_collection(handle: &DomHandle, tag: &'static str) -> JsValue {
    live_query::collection(handle, move |el| el.tag == tag)
}

fn link_collection(handle: &DomHandle) -> JsValue {
    live_query::collection(handle, |el| {
        matches!(el.tag.as_str(), "a" | "area") && el.attrs.contains_key("href")
    })
}
