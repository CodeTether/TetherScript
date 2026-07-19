use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{ResultValue, Value};

fn string(value: &str) -> Value {
    Value::Str(Rc::new(value.into()))
}

fn context(value: Value) -> Value {
    Value::Map(Rc::new(RefCell::new(HashMap::from([(
        "value".into(),
        value,
    )]))))
}

fn render(args: &[Value]) -> ResultValue {
    let Value::Result(result) = super::call(args).unwrap() else {
        panic!("tera_render must return Result");
    };
    result.as_ref().clone()
}

#[test]
fn autoescapes_html_by_default() {
    let args = [string("{{ value }}"), context(string("<tag>"))];
    let ResultValue::Ok(Value::Str(output)) = render(&args) else {
        panic!("render should succeed");
    };
    assert_eq!(output.as_str(), "&lt;tag&gt;");
}

#[test]
fn can_disable_autoescaping() {
    let args = [
        string("{{ value }}"),
        context(string("<tag>")),
        Value::Bool(false),
    ];
    let ResultValue::Ok(Value::Str(output)) = render(&args) else {
        panic!("render should succeed");
    };
    assert_eq!(output.as_str(), "<tag>");
}

#[test]
fn reports_unsupported_value_path() {
    let bytes = Value::Bytes(Rc::new(RefCell::new(vec![1])));
    let ResultValue::Err(error) = render(&[string(""), context(bytes)]) else {
        panic!("bytes context should fail");
    };
    assert!(error.contains("context.value"), "{error}");
}
