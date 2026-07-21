//! Supervised-process stream and pressure tests.

use std::thread;
use std::time::{Duration, Instant};

use crate::value::resource::OwnedResource;
use crate::value::Value;

use super::{error, ok};

fn shell(script: &str, capacity: usize) -> OwnedResource {
    let (command, arguments) = if cfg!(windows) {
        ("cmd", vec!["/C".into(), script.into()])
    } else {
        ("sh", vec!["-c".into(), script.into()])
    };
    OwnedResource::child_process_bounded(command, &arguments, capacity).unwrap()
}

fn wait_for_eof(child: &mut OwnedResource, method: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.call(method, &[]).unwrap() != Value::Bool(true) {
        assert!(Instant::now() < deadline, "{method} did not reach EOF");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn process_captures_stdout_and_stderr() {
    let script = if cfg!(windows) {
        "echo stdout-line & echo stderr-line 1>&2"
    } else {
        "printf stdout-line; printf stderr-line >&2"
    };
    let mut child = shell(script, 64);
    ok(child.call("wait", &[]).unwrap());
    wait_for_eof(&mut child, "stdout_eof");
    wait_for_eof(&mut child, "stderr_eof");
    let stdout = ok(child.call("read_stdout", &[Value::Int(64)]).unwrap());
    let stderr = ok(child.call("read_stderr", &[Value::Int(64)]).unwrap());
    assert!(format!("{stdout}").contains("stdout-line"));
    assert!(format!("{stderr}").contains("stderr-line"));
}

#[test]
fn process_stdin_reports_bounded_backpressure() {
    let script = if cfg!(windows) {
        "more > nul"
    } else {
        "cat >/dev/null"
    };
    let mut child = shell(script, 4);
    let five = Value::Str(std::rc::Rc::new("12345".into()));
    assert!(error(child.call("write_stdin", &[five]).unwrap()).contains("backpressure"));
    ok(child.call("close_stdin", &[]).unwrap());
    ok(child.call("wait", &[]).unwrap());
}
