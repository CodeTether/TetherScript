use super::*;

type Init<'a> = Option<&'a JsValue>;
type Map = HashMap<String, JsValue>;

macro_rules! forward {
    ($name:ident, $module:ident::$func:ident) => {
        pub(crate) fn $name(map: &mut Map, init: Init<'_>) {
            $module::$func(map, init);
        }
    };
}

pub(crate) fn none(_: &mut Map, _: Init<'_>) {}
forward!(custom, misc::custom);
forward!(mouse, mouse::insert);
forward!(keyboard, keyboard::insert);
forward!(input, text::input);
forward!(submit, misc::submit);
forward!(focus, misc::focus);
forward!(storage, storage::insert);
forward!(clipboard, clipboard::insert);
forward!(pop_state, lifecycle::pop_state);
forward!(hash_change, lifecycle::hash_change);
forward!(page_transition, lifecycle::page_transition);
forward!(before_unload, lifecycle::before_unload);
forward!(progress, lifecycle::progress);

pub(crate) fn pointer_event(map: &mut Map, init: Init<'_>) {
    mouse::insert(map, init);
    pointer::insert(map, init);
}

pub(crate) fn wheel_event(map: &mut Map, init: Init<'_>) {
    mouse::insert(map, init);
    wheel::insert(map, init);
}
