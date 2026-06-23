//! Unit tests for terminal style helpers.

use crate::value::Value;

use super::{style, style_span, val};

#[test]
fn opens_and_resets_named_styles() {
    let attrs = val::map_value([
        ("fg".into(), val::strv("green")),
        ("bold".into(), Value::Bool(true)),
    ]);
    let Value::Str(open) = style::open_value(&[attrs]).unwrap() else {
        panic!("style open should return string");
    };
    assert_eq!(&*open, "\x1b[1;32m");
    assert_eq!(style::reset(), "\x1b[0m");
}

#[test]
fn renders_styled_span_text() {
    let span = val::map_value([
        ("text".into(), val::strv("ok")),
        ("fg".into(), val::strv("yellow")),
    ]);
    let rendered = style_span::render(&span).unwrap();
    assert_eq!(rendered, "\x1b[33mok\x1b[0m");
}
