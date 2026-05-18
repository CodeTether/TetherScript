use super::super::super::*;
use super::super::live_query;
use super::matcher;

pub(super) fn by_tag(handle: &DomHandle, tag: &str) -> JsValue {
    let query = tag.to_string();
    live_query::collection(handle, move |el| matcher::tag(el, &query))
}

pub(super) fn by_class(handle: &DomHandle, class_name: &str) -> JsValue {
    let want = matcher::tokens(class_name);
    live_query::collection(handle, move |el| matcher::classes(el, &want))
}

pub(super) fn by_name(handle: &DomHandle, name: &str) -> JsValue {
    let query = name.to_string();
    live_query::collection(handle, move |el| matcher::name(el, &query))
}
