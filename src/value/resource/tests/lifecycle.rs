//! Shared lifecycle and deadline tests.

use std::time::Duration;

use crate::value::resource::OwnedResource;

use super::error;

#[test]
fn close_and_cancel_are_observable_and_idempotent() {
    let mut channel = OwnedResource::channel(1).unwrap();
    channel.close();
    channel.close();
    assert!(channel.is_closed());
    assert!(error(channel.call("len", &[]).unwrap()).contains("closed"));

    let mut timer = OwnedResource::timer(Duration::from_secs(1));
    timer.cancel().unwrap();
    timer.cancel().unwrap();
    assert!(timer.is_cancelled());
    assert!(error(timer.call("ready", &[]).unwrap()).contains("cancelled"));
}

#[test]
fn elapsed_deadline_blocks_operations_until_cleared() {
    let mut writer = OwnedResource::response_writer(4).unwrap();
    writer.set_deadline_after(Duration::ZERO);
    assert!(writer.is_expired());
    assert!(error(writer.call("len", &[]).unwrap()).contains("deadline exceeded"));
    writer.clear_deadline();
    assert!(!writer.is_expired());
}
