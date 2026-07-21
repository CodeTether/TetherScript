//! File, child-process, and loopback TCP resource tests.

use std::path::Path;
use std::time::Duration;

use crate::value::resource::{OwnedResource, ResourceKind};
use crate::value::Value;

use super::ok;

#[test]
fn file_and_child_process_are_owned_handles() {
    let mut file = OwnedResource::file(Path::new("Cargo.toml"), "read").unwrap();
    assert_eq!(file.kind(), ResourceKind::File);
    assert!(matches!(
        ok(file.call("read", &[Value::Int(1)]).unwrap()),
        Value::Bytes(_)
    ));

    let (command, args): (&str, Vec<String>) = if cfg!(windows) {
        ("cmd", vec!["/C".into(), "exit".into(), "0".into()])
    } else {
        ("true", Vec::new())
    };
    let mut child = OwnedResource::child_process(command, &args).unwrap();
    child.set_deadline_after(Duration::from_secs(5));
    let status = ok(child.call("wait", &[]).unwrap());
    assert!(matches!(status, Value::Map(_)));
}

#[test]
fn loopback_listener_accepts_an_owned_stream() {
    let mut listener = OwnedResource::tcp_listener("127.0.0.1", 0).unwrap();
    let Value::Int(port) = ok(listener.call("port", &[]).unwrap()) else {
        panic!("listener port should be an int");
    };
    let stream =
        OwnedResource::tcp_stream("127.0.0.1", port as u16, Duration::from_secs(1)).unwrap();
    assert_eq!(stream.kind(), ResourceKind::TcpStream);
    let accepted = ok(listener.call("accept", &[]).unwrap());
    assert!(matches!(accepted, Value::Resource(_)));
}
