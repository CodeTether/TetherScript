//! Language factories for operating-system resources.

use std::path::Path;
use std::time::Duration;

use crate::value::Value;

use super::{args, factory, OwnedResource};

pub(super) fn file(values: &[Value]) -> Result<Value, String> {
    let path = args::string(&values[0], "resource.file path");
    let mode = args::string(&values[1], "resource.file mode");
    Ok(factory::resource(path.and_then(|path| {
        mode.and_then(|mode| OwnedResource::file(Path::new(&path), &mode))
    })))
}

pub(super) fn child(values: &[Value]) -> Result<Value, String> {
    let command = args::string(&values[0], "resource.child_process command");
    let arguments = args::strings(&values[1], "resource.child_process arguments");
    Ok(factory::resource(command.and_then(|command| {
        arguments.and_then(|arguments| OwnedResource::child_process(&command, &arguments))
    })))
}

pub(super) fn tcp_connect(values: &[Value]) -> Result<Value, String> {
    let host = args::string(&values[0], "resource.tcp_connect host");
    let port = factory::port(&values[1], "resource.tcp_connect port");
    let timeout = args::u64(&values[2], "resource.tcp_connect timeout");
    Ok(factory::resource(host.and_then(|host| {
        port.and_then(|port| {
            timeout.and_then(|milliseconds| {
                OwnedResource::tcp_stream(&host, port, Duration::from_millis(milliseconds))
            })
        })
    })))
}

pub(super) fn tcp_listen(values: &[Value]) -> Result<Value, String> {
    let host = args::string(&values[0], "resource.tcp_listen host");
    let port = factory::port(&values[1], "resource.tcp_listen port");
    Ok(factory::resource(host.and_then(|host| {
        port.and_then(|port| OwnedResource::tcp_listener(&host, port))
    })))
}
