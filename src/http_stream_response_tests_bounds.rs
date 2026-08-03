//! Unit tests for bound parsing and the generator return contract.
//!
//! Bounds and the return contract are the two rules a script author trips over
//! first, so they are asserted apart from header defaults.

use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{ResultValue, Value};

use super::payload;
use super::values::str_value;
use super::{Bounds, DEFAULT_MAX_DURATION_MS, DEFAULT_MAX_EVENTS, EVENT_CEILING};

#[test]
fn bounds_default_when_unspecified() {
    let defaulted = Bounds::parse(&HashMap::new()).unwrap();
    assert_eq!(defaulted.max_events, DEFAULT_MAX_EVENTS);
    let millis = defaulted.max_duration.as_millis() as u64;
    assert_eq!(millis, DEFAULT_MAX_DURATION_MS);
}

#[test]
fn an_oversized_bound_is_clamped_not_refused() {
    let huge = HashMap::from([("max_events".to_string(), Value::Int(1 << 40))]);
    assert_eq!(Bounds::parse(&huge).unwrap().max_events, EVENT_CEILING);
}

#[test]
fn a_zero_bound_is_refused_by_name() {
    let zero = HashMap::from([("max_duration_ms".to_string(), Value::Int(0))]);
    let error = Bounds::parse(&zero).unwrap_err();
    assert!(error.contains("max_duration_ms"), "{error}");
}

#[test]
fn a_non_int_bound_is_refused_by_name() {
    let wrong = HashMap::from([("max_events".to_string(), str_value("10"))]);
    let error = Bounds::parse(&wrong).unwrap_err();
    assert!(error.contains("max_events"), "{error}");
}

#[test]
fn generator_returns_are_narrow() {
    assert_eq!(payload::bytes(&Value::Nil).unwrap(), None);
    let framed = Some(b"data: hi\n\n".to_vec());
    assert_eq!(payload::bytes(&str_value("data: hi\n\n")).unwrap(), framed);
    let wrapped = Value::Result(Rc::new(ResultValue::Ok(str_value("data: hi\n\n"))));
    assert_eq!(payload::bytes(&wrapped).unwrap(), framed);
    let failed = Value::Result(Rc::new(ResultValue::Err("boom".into())));
    assert_eq!(payload::bytes(&failed).unwrap_err(), "boom");
    assert!(payload::bytes(&Value::Int(7)).is_err());
}

#[test]
fn a_nested_result_is_refused_rather_than_unwrapped() {
    let inner = Value::Result(Rc::new(ResultValue::Ok(str_value("data: hi\n\n"))));
    let outer = Value::Result(Rc::new(ResultValue::Ok(inner)));
    assert!(payload::bytes(&outer).is_err());
}
