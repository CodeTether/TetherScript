//! Responses model option normalization.

use crate::value::Value;

pub(super) fn request_model(opts: Option<&Value>) -> Result<(String, Option<String>), String> {
    let model = opts
        .and_then(|value| match value {
            Value::Map(map) => map.borrow().get("model").cloned(),
            _ => None,
        })
        .and_then(|value| match value {
            Value::Str(text) => Some(text.to_string()),
            _ => None,
        })
        .ok_or("provider.responses: model is required")?;
    Ok(service_tier_alias(&model))
}

fn service_tier_alias(model: &str) -> (String, Option<String>) {
    match model {
        "gpt-5.4-fast" => ("gpt-5.4".into(), Some("priority".into())),
        "gpt-5.5-fast" => ("gpt-5.5".into(), Some("priority".into())),
        _ => (model.into(), None),
    }
}
