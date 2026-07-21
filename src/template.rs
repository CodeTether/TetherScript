//! Tera template rendering for tetherscript values.

#[cfg(feature = "tera")]
mod convert;
#[cfg(not(feature = "tera"))]
mod disabled;
#[cfg(feature = "tera")]
mod render;
#[cfg(all(test, feature = "tera"))]
mod tests;

use std::rc::Rc;

use crate::value::{Env, NativeFn, NativeFunc, Value};

pub(crate) fn install(env: &mut Env) {
    env.define(
        "tera_render",
        Value::Native(Rc::new(NativeFn {
            name: "tera_render".into(),
            arity: None,
            func: NativeFunc::Pure(Box::new(render)),
        })),
        false,
    );
}

#[cfg(feature = "tera")]
fn render(args: &[Value]) -> Result<Value, String> {
    render::call(args)
}

#[cfg(not(feature = "tera"))]
fn render(args: &[Value]) -> Result<Value, String> {
    disabled::call(args)
}
