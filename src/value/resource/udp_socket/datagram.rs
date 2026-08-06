//! The `{ bytes, from }` map a received datagram becomes.
//!
//! UDP is connectionless, so the sender address is part of the payload rather
//! than a property of the socket; returning them together keeps a caller from
//! having to correlate two calls.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

/// Build the datagram map from received `bytes` and a sender `from` address.
pub(super) fn value(bytes: Vec<u8>, from: &str) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "bytes".to_string(),
        Value::Bytes(Rc::new(RefCell::new(bytes))),
    );
    map.insert("from".to_string(), Value::Str(Rc::new(from.to_string())));
    Value::Map(Rc::new(RefCell::new(map)))
}
