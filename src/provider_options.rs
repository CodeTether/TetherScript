//! OpenAI-compatible provider request option handling.

use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(crate) fn apply(
    body: &mut HashMap<String, Value>,
    opts: &HashMap<String, Value>,
    max_tokens: u64,
) -> String {
    let model = copy_model(body, opts);
    copy_capped_int(body, opts, "max_tokens", max_tokens);
    copy_capped_int(body, opts, "max_completion_tokens", max_tokens);
    copy_keys(
        body,
        opts,
        &[
            "temperature",
            "stream",
            "top_p",
            "reasoning_effort",
            "clear_thinking",
            "tools",
            "tool_choice",
            "parallel_tool_calls",
        ],
    );
    model
}

fn copy_model(body: &mut HashMap<String, Value>, opts: &HashMap<String, Value>) -> String {
    match opts.get("model") {
        Some(Value::Str(name)) => {
            body.insert("model".into(), Value::Str(Rc::new(name.to_string())));
            name.to_string()
        }
        _ => String::new(),
    }
}

fn copy_capped_int(
    body: &mut HashMap<String, Value>,
    opts: &HashMap<String, Value>,
    key: &str,
    max_tokens: u64,
) {
    if let Some(Value::Int(value)) = opts.get(key) {
        let capped = if max_tokens > 0 {
            (*value).min(max_tokens as i64)
        } else {
            *value
        };
        body.insert(key.into(), Value::Int(capped));
    }
}

fn copy_keys(body: &mut HashMap<String, Value>, opts: &HashMap<String, Value>, keys: &[&str]) {
    for key in keys {
        if let Some(value) = opts.get(*key) {
            body.insert((*key).into(), value.clone());
        }
    }
}
