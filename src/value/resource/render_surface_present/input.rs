//! Native keyboard and mouse state snapshots.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};

use crate::value::Value;

pub(super) fn snapshot(window: &Window) -> Value {
    let mut fields = HashMap::new();
    let mouse = window.get_mouse_pos(MouseMode::Clamp);
    fields.insert("mouse_x".into(), coordinate(mouse.map(|point| point.0)));
    fields.insert("mouse_y".into(), coordinate(mouse.map(|point| point.1)));
    fields.insert(
        "left_down".into(),
        Value::Bool(window.get_mouse_down(MouseButton::Left)),
    );
    fields.insert(
        "middle_down".into(),
        Value::Bool(window.get_mouse_down(MouseButton::Middle)),
    );
    fields.insert(
        "right_down".into(),
        Value::Bool(window.get_mouse_down(MouseButton::Right)),
    );
    fields.insert("keys_down".into(), keys(window.get_keys()));
    fields.insert(
        "keys_pressed".into(),
        keys(window.get_keys_pressed(KeyRepeat::No)),
    );
    fields.insert("keys_released".into(), keys(window.get_keys_released()));
    Value::Map(Rc::new(RefCell::new(fields)))
}

fn coordinate(value: Option<f32>) -> Value {
    value.map_or(Value::Nil, |value| Value::Int(value as i64))
}

fn keys(values: Vec<Key>) -> Value {
    let values = values
        .into_iter()
        .map(|key| Value::Str(Rc::new(format!("{key:?}").to_ascii_lowercase())))
        .collect();
    Value::List(Rc::new(RefCell::new(values)))
}
