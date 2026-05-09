use super::*;

#[path = "events/base.rs"]
mod base;
#[path = "events/class.rs"]
mod class;
#[path = "events/data_transfer.rs"]
mod data_transfer;
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
#[path = "events/tests_data_transfer.rs"]
mod tests_data_transfer;
#[cfg(test)]
#[path = "events/tests_interactions.rs"]
mod tests_interactions;
#[cfg(test)]
#[path = "events/tests_legacy.rs"]
mod tests_legacy;
#[cfg(test)]
#[path = "events/tests_lifecycle.rs"]
mod tests_lifecycle;
#[cfg(test)]
#[path = "events/tests_message_event.rs"]
mod tests_message_event;
#[cfg(test)]
#[path = "events/tests_storage_clipboard.rs"]
mod tests_storage_clipboard;

pub(super) fn install(window: &mut HashMap<String, JsValue>) {
    data_transfer::install(window);
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
