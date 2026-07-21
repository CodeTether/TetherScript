//! Native function entries for the language `resource` namespace.

use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{NativeFn, NativeFunc, Value};

pub(super) type Factory = fn(&[Value]) -> Result<Value, String>;

pub(super) fn insert(
    module: &mut HashMap<String, Value>,
    name: &str,
    arity: usize,
    factory: Factory,
) {
    let qualified = format!("resource.{name}");
    module.insert(
        name.into(),
        Value::Native(Rc::new(NativeFn {
            name: qualified,
            arity: Some(arity),
            func: NativeFunc::Pure(Box::new(factory)),
        })),
    );
}
