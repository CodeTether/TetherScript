use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use tetherscript::value::Value;

pub(super) fn row(name: &str, value: String) -> Value {
    let row = HashMap::from([(name.into(), Value::Str(Rc::new(value)))]);
    let row = Value::Map(Rc::new(RefCell::new(row)));
    Value::List(Rc::new(RefCell::new(vec![row])))
}
