#[derive(Clone, Copy)]
pub(super) enum EventClass {
    Event,
    Custom,
    Mouse,
    Keyboard,
    Input,
    Submit,
    Focus,
    Pointer,
    Wheel,
}

pub(super) fn all() -> [EventClass; 9] {
    [
        EventClass::Event,
        EventClass::Custom,
        EventClass::Mouse,
        EventClass::Keyboard,
        EventClass::Input,
        EventClass::Submit,
        EventClass::Focus,
        EventClass::Pointer,
        EventClass::Wheel,
    ]
}

impl EventClass {
    pub(super) fn name(self) -> &'static str {
        match self {
            EventClass::Event => "Event",
            EventClass::Custom => "CustomEvent",
            EventClass::Mouse => "MouseEvent",
            EventClass::Keyboard => "KeyboardEvent",
            EventClass::Input => "InputEvent",
            EventClass::Submit => "SubmitEvent",
            EventClass::Focus => "FocusEvent",
            EventClass::Pointer => "PointerEvent",
            EventClass::Wheel => "WheelEvent",
        }
    }
}
