use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub(in crate::process_control) fn value(processes: Vec<(i64, String)>) -> Value {
    let values = processes
        .into_iter()
        .map(|(pid, name)| {
            let map = HashMap::from([
                ("pid".into(), Value::Int(pid)),
                ("name".into(), Value::Str(Rc::new(name))),
            ]);
            Value::Map(Rc::new(RefCell::new(map)))
        })
        .collect();
    Value::List(Rc::new(RefCell::new(values)))
}
