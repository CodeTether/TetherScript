//! Language factories for bounded in-memory resources.

use std::time::Duration;

use crate::value::Value;

use super::{args, factory, OwnedResource};

pub(super) fn request_body(values: &[Value]) -> Result<Value, String> {
    let body = args::bytes(&values[0], "resource.request_body body");
    let capacity = args::usize(&values[1], "resource.request_body capacity");
    Ok(factory::resource(body.and_then(|body| {
        capacity.and_then(|capacity| OwnedResource::request_body(body, capacity))
    })))
}

pub(super) fn response_writer(values: &[Value]) -> Result<Value, String> {
    Ok(factory::resource(
        args::usize(&values[0], "resource.response_writer capacity")
            .and_then(OwnedResource::response_writer),
    ))
}

pub(super) fn task(_values: &[Value]) -> Result<Value, String> {
    Ok(factory::direct(OwnedResource::task()))
}

pub(super) fn timer(values: &[Value]) -> Result<Value, String> {
    let milliseconds = args::u64(&values[0], "resource.timer delay")?;
    Ok(factory::direct(OwnedResource::timer(
        Duration::from_millis(milliseconds),
    )))
}

pub(super) fn channel(values: &[Value]) -> Result<Value, String> {
    Ok(factory::resource(
        args::usize(&values[0], "resource.channel capacity").and_then(OwnedResource::channel),
    ))
}
