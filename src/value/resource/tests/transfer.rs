//! Transfer enforcement for resource-retaining operations.

use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::value::Value;

use super::{error, ok};
use crate::value::resource::OwnedResource;

fn timer_value() -> Value {
    Value::Resource(Rc::new(RefCell::new(OwnedResource::timer(Duration::ZERO))))
}

#[test]
fn channel_requires_move_before_retaining_a_resource() {
    let owner = timer_value();
    let borrowed = owner.clone();
    let mut channel = OwnedResource::channel(1).unwrap();

    let rejected = channel.call("send", &[borrowed.clone()]).unwrap();
    let message = error(rejected);
    assert!(message.contains("channel.send"));
    assert!(message.contains("use `move`"));

    drop(borrowed);
    let transferred = vec![owner];
    ok(channel.call("send", &transferred).unwrap());
}

#[test]
fn task_requires_move_before_retaining_a_resource() {
    let owner = timer_value();
    let borrowed = owner.clone();
    let mut task = OwnedResource::task();

    let rejected = task.call("complete", &[borrowed.clone()]).unwrap();
    let message = error(rejected);
    assert!(message.contains("task.complete"));
    assert!(message.contains("timer resource"));

    drop(borrowed);
    let transferred = vec![owner];
    ok(task.call("complete", &transferred).unwrap());
    drop(transferred);
    assert!(matches!(
        ok(task.call("result", &[]).unwrap()),
        Value::Resource(_)
    ));
    assert!(error(task.call("result", &[]).unwrap()).contains("consumed"));
}
