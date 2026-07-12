use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

#[test]
fn forwards_codex_reasoning_and_service_tier() {
    let mut body = HashMap::new();
    let mut opts = HashMap::new();
    opts.insert("model".into(), text("gpt-5.6-sol"));
    opts.insert("reasoning_effort".into(), text("high"));
    opts.insert("service_tier".into(), text("priority"));

    super::apply(&mut body, &opts, 0);

    assert_eq!(body.get("model"), Some(&text("gpt-5.6-sol")));
    assert_eq!(body.get("reasoning_effort"), Some(&text("high")));
    assert_eq!(body.get("service_tier"), Some(&text("priority")));
}

fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.into()))
}
