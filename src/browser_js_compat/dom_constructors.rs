use super::*;

#[path = "dom_constructors/image.rs"]
mod image;
#[path = "dom_constructors/node.rs"]
mod node;
#[path = "dom_constructors/option.rs"]
mod option;
#[path = "dom_constructors/unsupported.rs"]
mod unsupported;

#[cfg(test)]
#[path = "tests_dom_constructor_globals.rs"]
mod tests_dom_constructor_globals;
#[cfg(test)]
#[path = "tests_dom_constructors.rs"]
mod tests_dom_constructors;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    window.insert("Node".into(), node::create());
    window.insert("Image".into(), native("Image", None, image::create));
    window.insert("Option".into(), native("Option", None, option::create));
    unsupported::install(window);
}

fn string_arg(args: &[JsValue], index: usize, default: &str) -> String {
    match args.get(index).unwrap_or(&JsValue::Undefined) {
        JsValue::Undefined | JsValue::Null => default.into(),
        value => value.display(),
    }
}

fn bool_arg(args: &[JsValue], index: usize, default: bool) -> bool {
    args.get(index).map(JsValue::truthy).unwrap_or(default)
}

fn number_arg(args: &[JsValue], index: usize) -> Option<f64> {
    match args.get(index) {
        Some(JsValue::Number(value)) if value.is_finite() => Some(value.max(0.0).trunc()),
        Some(value) => value
            .display()
            .parse::<f64>()
            .ok()
            .map(|n| n.max(0.0).trunc()),
        None => None,
    }
}

fn object(value: &JsValue) -> Option<Rc<RefCell<HashMap<String, JsValue>>>> {
    match value {
        JsValue::Object(obj) => Some(obj.clone()),
        _ => None,
    }
}
