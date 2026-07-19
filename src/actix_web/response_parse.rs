//! Validation of tetherscript response maps.

use ::actix_web::http::StatusCode;

use crate::value::{ResultValue, Value};

use super::error::ActixPluginError;
use super::response::ResponseData;

pub(super) fn parse(value: Value) -> Result<ResponseData, ActixPluginError> {
    let value = unwrap_result(value)?;
    let Value::Map(fields) = value else {
        return ActixPluginError::reject("hook must return a map");
    };
    let fields = fields.borrow();
    let status = status(fields.get("status"))?;
    let headers = super::response_headers::parse(fields.get("headers"))?;
    let body = body(fields.get("body"))?;
    Ok(ResponseData {
        status,
        headers,
        body,
    })
}

fn unwrap_result(value: Value) -> Result<Value, ActixPluginError> {
    match value {
        Value::Result(result) => match result.as_ref() {
            ResultValue::Ok(value) => Ok(value.clone()),
            ResultValue::Err(error) => ActixPluginError::reject(error),
        },
        value => Ok(value),
    }
}

fn status(value: Option<&Value>) -> Result<u16, ActixPluginError> {
    let status = match value {
        None => 200,
        Some(Value::Int(value)) => u16::try_from(*value)
            .map_err(|_| ActixPluginError::invalid("status must be 100..=599"))?,
        Some(_) => return ActixPluginError::reject("status must be int"),
    };
    StatusCode::from_u16(status)
        .map_err(|_| ActixPluginError::invalid("status must be 100..=599"))?;
    Ok(status)
}

fn body(value: Option<&Value>) -> Result<Vec<u8>, ActixPluginError> {
    match value {
        None | Some(Value::Nil) => Ok(Vec::new()),
        Some(Value::Str(value)) => Ok(value.as_bytes().to_vec()),
        Some(Value::Bytes(value)) => Ok(value.borrow().clone()),
        Some(_) => ActixPluginError::reject("body must be str, bytes, or nil"),
    }
}
