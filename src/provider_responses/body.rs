//! Responses request body construction.

#[path = "tool_options.rs"]
mod tool_options;

use crate::value::Value;

use super::messages;
use super::model;

pub(super) fn build(args: &[Value], max_tokens: u64) -> Result<String, String> {
    let messages = args
        .first()
        .ok_or("provider.responses: expected messages argument")?;
    let opts = args.get(1);
    let (model, tier) = model::request_model(opts)?;
    let mut parts = vec![
        format!("\"model\":{}", string(&model)?),
        format!(
            "\"instructions\":{}",
            string(&messages::instructions(messages))?
        ),
        format!("\"input\":{}", messages::input(messages)?),
        "\"stream\":true".into(),
        "\"store\":false".into(),
    ];
    tool_options::push(&mut parts, opts)?;
    if let Some(tier) = tier {
        parts.push(format!("\"service_tier\":{}", string(&tier)?));
    }
    if max_tokens > 0 {
        parts.push(format!("\"max_output_tokens\":{}", max_tokens));
    }
    Ok(format!("{{{}}}", parts.join(",")))
}

pub(super) fn string(text: &str) -> Result<String, String> {
    crate::json::encode_to_string(&Value::Str(std::rc::Rc::new(text.to_string())))
}
