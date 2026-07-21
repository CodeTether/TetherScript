//! Bounded HTTP, task, timer, and channel resource tests.

use std::rc::Rc;
use std::time::Duration;

use crate::value::Value;

use super::{error, ok};
use crate::value::resource::OwnedResource;

#[test]
fn bounded_buffers_report_pressure_without_losing_data() {
    let mut request = OwnedResource::request_body(b"body".to_vec(), 4).unwrap();
    let body = ok(request.call("read", &[Value::Int(4)]).unwrap());
    assert_eq!(
        body,
        Value::Bytes(Rc::new(std::cell::RefCell::new(b"body".to_vec())))
    );

    let mut response = OwnedResource::response_writer(4).unwrap();
    ok(response
        .call("write", &[Value::Str(Rc::new("done".into()))])
        .unwrap());
    let pressure = response
        .call("write", &[Value::Str(Rc::new("!".into()))])
        .unwrap();
    assert!(error(pressure).contains("backpressure"));
}

#[test]
fn tasks_timers_and_channels_have_explicit_readiness() {
    let mut task = OwnedResource::task();
    assert!(error(task.call("result", &[]).unwrap()).contains("pending"));
    ok(task.call("complete", &[Value::Int(7)]).unwrap());
    assert_eq!(ok(task.call("result", &[]).unwrap()), Value::Int(7));

    let mut timer = OwnedResource::timer(Duration::ZERO);
    assert_eq!(timer.call("ready", &[]).unwrap(), Value::Bool(true));
    let mut channel = OwnedResource::channel(1).unwrap();
    ok(channel.call("send", &[Value::Int(1)]).unwrap());
    assert!(error(channel.call("send", &[Value::Int(2)]).unwrap()).contains("backpressure"));
    assert_eq!(ok(channel.call("recv", &[]).unwrap()), Value::Int(1));
}
