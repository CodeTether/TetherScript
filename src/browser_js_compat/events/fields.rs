use super::*;

#[path = "fields/keyboard.rs"]
mod keyboard;
#[path = "fields/misc.rs"]
mod misc;
#[path = "fields/mouse.rs"]
mod mouse;
#[path = "fields/pointer.rs"]
mod pointer;
#[path = "fields/text.rs"]
mod text;
#[path = "fields/wheel.rs"]
mod wheel;

pub(super) fn insert(
    map: &mut HashMap<String, JsValue>,
    event_class: class::EventClass,
    init: Option<&JsValue>,
) {
    match event_class {
        class::EventClass::Custom => misc::custom(map, init),
        class::EventClass::Mouse => mouse::insert(map, init),
        class::EventClass::Keyboard => keyboard::insert(map, init),
        class::EventClass::Input => text::input(map, init),
        class::EventClass::Submit => misc::submit(map, init),
        class::EventClass::Focus => misc::focus(map, init),
        class::EventClass::Pointer => {
            mouse::insert(map, init);
            pointer::insert(map, init);
        }
        class::EventClass::Wheel => {
            mouse::insert(map, init);
            wheel::insert(map, init);
        }
        class::EventClass::Event => {}
    }
}
