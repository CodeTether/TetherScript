//! Recursive transfer validation for resource-owning aggregates.

use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use crate::value::{resource::transfer, ResultValue, Value};

use crate::value::resource::OwnedResource;

fn timer() -> Value {
    Value::Resource(Rc::new(RefCell::new(OwnedResource::timer(Duration::ZERO))))
}

#[test]
fn borrowed_resource_aggregates_are_rejected() {
    let values = [
        Value::List(Rc::new(RefCell::new(vec![timer()]))),
        Value::Map(Rc::new(RefCell::new(HashMap::from([(
            "timer".into(),
            timer(),
        )])))),
        Value::Result(Rc::new(ResultValue::Ok(timer()))),
    ];
    for value in values {
        let borrowed = value.clone();
        let error = transfer::validate(&borrowed, "binding `alias`").unwrap_err();
        assert!(error.contains("containing timer resource"), "{error}");
        drop(borrowed);
        transfer::validate(&value, "binding `moved`").unwrap();
    }
}

#[test]
fn cyclic_aggregates_are_scanned_once() {
    let values = Rc::new(RefCell::new(Vec::new()));
    let list = Value::List(values.clone());
    values.borrow_mut().push(list.clone());
    values.borrow_mut().push(timer());

    let error = transfer::validate(&list, "binding `cycle`").unwrap_err();
    assert!(error.contains("list containing timer resource"), "{error}");
}
