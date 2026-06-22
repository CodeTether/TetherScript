//! Unit tests for dependency-free TUI helpers.

use crate::value::Value;

use super::{ansi, render, val};

#[test]
fn render_draws_agent_items() {
    let item = val::map_value([
        ("kind".into(), val::strv("agent")),
        ("name".into(), val::strv("planner")),
        ("text".into(), val::strv("ready")),
    ]);
    let view = val::map_value([
        ("width".into(), Value::Int(32)),
        ("height".into(), Value::Int(5)),
        ("title".into(), val::strv("bus")),
        ("status".into(), val::strv("idle")),
        (
            "items".into(),
            Value::List(std::rc::Rc::new(std::cell::RefCell::new(vec![item]))),
        ),
    ]);
    let Value::Str(out) = render::render(&[view]).unwrap() else {
        panic!("render should return string");
    };
    assert!(out.contains("[agent] planner: ready"));
}

#[test]
fn ansi_move_to_uses_one_based_coordinates() {
    let Value::Str(out) = ansi::move_to(&[Value::Int(2), Value::Int(3)]).unwrap() else {
        panic!("move_to should return string");
    };
    assert_eq!(&*out, "\x1b[2;3H");
}
