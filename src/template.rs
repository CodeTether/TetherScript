//! Tera template rendering for tetherscript values.

mod convert;
#[cfg(test)]
mod tests;

use std::rc::Rc;

use crate::value::{Env, NativeFn, NativeFunc, ResultValue, Value};

pub(crate) fn install(env: &mut Env) {
    env.define(
        "tera_render",
        Value::Native(Rc::new(NativeFn {
            name: "tera_render".into(),
            arity: None,
            func: NativeFunc::Pure(Box::new(call)),
        })),
        false,
    );
}

fn call(args: &[Value]) -> Result<Value, String> {
    let result = arguments(args).and_then(|(source, context, autoescape)| {
        let context = convert::context(context)?;
        tera::Tera::one_off(source, &context, autoescape)
            .map_err(|error| format!("tera_render: {error}"))
    });
    let result = match result {
        Ok(output) => ResultValue::Ok(Value::Str(Rc::new(output))),
        Err(error) => ResultValue::Err(error),
    };
    Ok(Value::Result(Rc::new(result)))
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
