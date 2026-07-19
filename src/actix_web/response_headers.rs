//! Validation of response headers returned by scripts.

use ::actix_web::http::header::{HeaderName, HeaderValue};

use crate::value::Value;

use super::error::ActixPluginError;

pub(super) fn parse(
    value: Option<&Value>,
) -> Result<Vec<(HeaderName, HeaderValue)>, ActixPluginError> {
    let Some(Value::Map(values)) = value else {
        return if value.is_none() {
            Ok(vec![])
        } else {
            ActixPluginError::reject("headers must be map")
        };
    };
    values
        .borrow()
        .iter()
        .map(|(key, value)| match value {
            Value::Str(value) => {
                let name = HeaderName::try_from(key.as_str()).map_err(|_| {
                    ActixPluginError::invalid(&format!("invalid header name `{key}`"))
                })?;
                let value = HeaderValue::try_from(value.as_str())
                    .map_err(|_| ActixPluginError::invalid("invalid header value"))?;
                Ok((name, value))
            }
            _ => ActixPluginError::reject("header values must be str"),
        })
        .collect()
}
