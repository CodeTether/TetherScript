use super::*;

pub(super) fn install(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    handle: &DomHandle,
) {
    let mut map = object.borrow_mut();
    for name in [
        "clientWidth",
        "clientHeight",
        "scrollWidth",
        "scrollHeight",
        "offsetLeft",
        "offsetTop",
        "clientLeft",
        "clientTop",
        "scrollLeft",
        "scrollTop",
    ] {
        let handle = handle.clone();
        let object = object.clone();
        map.insert(
            format!("__get:{name}"),
            native(name, Some(0), move |_| {
                let handle = access::current_handle(&object, &handle);
                Ok(JsValue::Number(value(&handle, name) as f64))
            }),
        );
    }
    drop(map);
    super::setters::install(object, handle);
}

fn value(handle: &DomHandle, name: &str) -> i64 {
    let geometry = geometry::measure(handle);
    match name {
        "clientWidth" => geometry.client_width,
        "clientHeight" => geometry.client_height,
        "scrollWidth" => geometry.scroll_width,
        "scrollHeight" => geometry.scroll_height,
        "offsetLeft" => geometry.x,
        "offsetTop" => geometry.y,
        "clientLeft" => geometry.client_left,
        "clientTop" => geometry.client_top,
        "scrollLeft" => apply::current(handle).left,
        "scrollTop" => apply::current(handle).top,
        _ => 0,
    }
}
