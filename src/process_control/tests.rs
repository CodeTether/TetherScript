use std::process::{Command, Stdio};

use super::*;

#[test]
fn process_list_contains_test_process() {
    let Value::Result(result) = list() else {
        panic!("expected Result")
    };
    let ResultValue::Ok(Value::List(processes)) = result.as_ref() else {
        panic!("process_list failed")
    };
    let own_pid = std::process::id() as i64;
    assert!(processes.borrow().iter().any(|value| matches!(value,
        Value::Map(map) if map.borrow().get("pid") == Some(&Value::Int(own_pid)))));
}

#[test]
fn process_kill_force_terminates_child() {
    #[cfg(windows)]
    let mut child = Command::new("cmd")
        .args(["/C", "ping -n 30 127.0.0.1 >NUL"])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    #[cfg(not(windows))]
    let mut child = Command::new("sleep")
        .arg("30")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let value = kill(&[Value::Int(child.id() as i64), Value::Bool(true)]);
    assert!(
        matches!(value, Value::Result(result) if matches!(result.as_ref(), ResultValue::Ok(Value::Nil)))
    );
    assert!(child.wait().unwrap().code() != Some(0));
}

#[test]
fn process_kill_rejects_invalid_pid() {
    let value = kill(&[Value::Int(-1)]);
    assert!(matches!(value, Value::Result(result)
        if matches!(result.as_ref(), ResultValue::Err(error) if error.contains("pid must be between"))));
}
