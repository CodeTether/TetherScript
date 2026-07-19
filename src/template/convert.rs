//! Conversion from tetherscript values to Tera context values.

use crate::value::Value;

pub(super) fn context(value: &Value) -> Result<tera::Context, String> {
    if !matches!(value, Value::Map(_)) {
        return Err(format!(
            "tera_render: context must be map, got {}",
            value.type_name()
        ));
    }
    tera::Context::from_serialize(convert(value, "context")?)
        .map_err(|error| format!("tera_render: invalid context: {error}"))
}

fn convert(value: &Value, path: &str) -> Result<tera::Value, String> {
    match value {
        Value::Nil => Ok(tera::Value::Null),
        Value::Int(value) => Ok(tera::Value::Number((*value).into())),
        Value::Float(value) => tera::to_value(value)
            .map_err(|error| format!("tera_render: invalid value at {path}: {error}")),
        Value::Bool(value) => Ok(tera::Value::Bool(*value)),
        Value::Str(value) => Ok(tera::Value::String(value.to_string())),
        Value::List(values) => values
            .borrow()
            .iter()
            .enumerate()
            .map(|(index, value)| convert(value, &format!("{path}[{index}]")))
            .collect::<Result<Vec<_>, _>>()
            .map(tera::Value::Array),
        Value::Map(values) => values
            .borrow()
            .iter()
            .map(|(key, value)| Ok((key.clone(), convert(value, &format!("{path}.{key}"))?)))
            .collect::<Result<_, String>>()
            .map(tera::Value::Object),
        other => Err(format!(
            "tera_render: unsupported {} at {path}",
            other.type_name()
        )),
    }
}
