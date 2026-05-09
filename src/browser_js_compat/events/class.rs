use super::*;

type FieldInit = fn(&mut HashMap<String, JsValue>, Option<&JsValue>);

#[derive(Clone, Copy)]
pub(super) struct EventClass {
    name: &'static str,
    fields: FieldInit,
    custom_init: bool,
}

impl EventClass {
    const fn new(name: &'static str, fields: FieldInit, custom_init: bool) -> Self {
        Self {
            name,
            fields,
            custom_init,
        }
    }

    pub(super) fn name(self) -> &'static str {
        self.name
    }

    pub(super) fn insert_fields(self, map: &mut HashMap<String, JsValue>, init: Option<&JsValue>) {
        (self.fields)(map, init);
    }

    pub(super) fn has_custom_init(self) -> bool {
        self.custom_init
    }
}

pub(super) fn all() -> [EventClass; 16] {
    use super::fields::constructors as init;

    [
        EventClass::new("Event", init::none, false),
        EventClass::new("CustomEvent", init::custom, true),
        EventClass::new("MouseEvent", init::mouse, false),
        EventClass::new("KeyboardEvent", init::keyboard, false),
        EventClass::new("InputEvent", init::input, false),
        EventClass::new("SubmitEvent", init::submit, false),
        EventClass::new("FocusEvent", init::focus, false),
        EventClass::new("PointerEvent", init::pointer_event, false),
        EventClass::new("WheelEvent", init::wheel_event, false),
        EventClass::new("StorageEvent", init::storage, false),
        EventClass::new("ClipboardEvent", init::clipboard, false),
        EventClass::new("PopStateEvent", init::pop_state, false),
        EventClass::new("HashChangeEvent", init::hash_change, false),
        EventClass::new("PageTransitionEvent", init::page_transition, false),
        EventClass::new("BeforeUnloadEvent", init::before_unload, false),
        EventClass::new("ProgressEvent", init::progress, false),
    ]
}
