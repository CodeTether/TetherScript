//! Tera-backed rendering enabled by the optional `tera` feature.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

pub(super) fn call(args: &[Value]) -> Result<Value, String> {
    let result = arguments(args).and_then(|(source, context, autoescape)| {
        let context = super::convert::context(context)?;
        tera::Tera::one_off(source, &context, autoescape)
            .map_err(|error| format!("tera_render: {error}"))
    });
    Ok(Value::Result(Rc::new(match result {
        Ok(output) => ResultValue::Ok(Value::Str(Rc::new(output))),
        Err(error) => ResultValue::Err(error),
    })))
}

fn arguments(args: &[Value]) -> Result<(&str, &Value, bool), String> {
    let [Value::Str(source), context, rest @ ..] = args else {
        return Err("tera_render: expected template string and context map".into());
    };
    match rest {
        [] => Ok((source, context, true)),
        [Value::Bool(autoescape)] => Ok((source, context, *autoescape)),
        [_] => Err("tera_render: autoescape must be bool".into()),
        _ => Err("tera_render: expected 2 or 3 arguments".into()),
    }
}
