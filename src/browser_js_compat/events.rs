use super::*;

#[path = "events/base.rs"]
mod base;
#[path = "events/class.rs"]
mod class;
#[path = "events/fields.rs"]
mod fields;
#[path = "events/methods.rs"]
mod methods;
#[path = "events/object.rs"]
mod object;

#[cfg(test)]
#[path = "events/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "events/tests_legacy.rs"]
mod tests_legacy;
#[cfg(test)]
#[path = "events/tests_storage_clipboard.rs"]
mod tests_storage_clipboard;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    for event_class in class::all() {
        let name = event_class.name();
        window.insert(
            name.into(),
            native(name, None, move |args| {
                Ok(object::create(event_class, args))
            }),
        );
    }
}
