use super::EventClass;

pub(super) const COUNT: usize = 19;

pub(super) fn all() -> [EventClass; COUNT] {
    use super::super::fields::constructors as init;

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
        EventClass::new("DragEvent", init::drag_event, false),
        EventClass::new("CompositionEvent", init::composition, false),
        EventClass::new("TouchEvent", init::touch, false),
        EventClass::new("StorageEvent", init::storage, false),
        EventClass::new("ClipboardEvent", init::clipboard, false),
        EventClass::new("PopStateEvent", init::pop_state, false),
        EventClass::new("HashChangeEvent", init::hash_change, false),
        EventClass::new("PageTransitionEvent", init::page_transition, false),
        EventClass::new("BeforeUnloadEvent", init::before_unload, false),
        EventClass::new("ProgressEvent", init::progress, false),
    ]
}
