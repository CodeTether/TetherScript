use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use tetherscript::capability::Authority;
use tetherscript::computer_cap::ComputerAuthority;
use tetherscript::value::{Runtime, Value};

struct NoopRuntime;

impl Runtime for NoopRuntime {
    fn invoke(&mut self, _callee: &Value, _args: &[Value]) -> Result<Value, String> {
        Ok(Value::Nil)
    }
}

struct FakeBridge {
    endpoint: String,
    body: Arc<Mutex<String>>,
    handle: thread::JoinHandle<()>,
}

impl FakeBridge {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/computer", listener.local_addr().unwrap());
        let body = Arc::new(Mutex::new(String::new()));
        let body_clone = body.clone();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut data = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0);
                data.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&data);
                if let Some(split) = text.find("\r\n\r\n") {
                    let content_length = text[..split]
                        .lines()
                        .find_map(|line| line.strip_prefix("Content-Length: "))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    if data.len() >= split + 4 + content_length {
                        break;
                    }
                }
            }
            *body_clone.lock().unwrap() = String::from_utf8_lossy(&data).into_owned();
            let response = r#"{"ok":true,"value":{"clicked":true}}"#;
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            );
            stream.write_all(reply.as_bytes()).unwrap();
        });
        Self {
            endpoint,
            body,
            handle,
        }
    }
}

fn text(value: &str) -> Value {
    Value::Str(Rc::new(value.to_string()))
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(Rc::new(RefCell::new(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )))
}

fn invoke(auth: &Rc<dyn Authority>, method: &str, args: &[Value]) -> Result<Value, String> {
    auth.invoke(&mut NoopRuntime, method, args)
}

fn map_number(value: &Value, key: &str) -> f64 {
    let Value::Map(values) = value else {
        panic!("expected map")
    };
    match values.borrow().get(key).unwrap() {
        Value::Int(value) => *value as f64,
        Value::Float(value) => *value,
        other => panic!("expected number, got {}", other.type_name()),
    }
}

fn set_cursor(auth: &Rc<dyn Authority>, name: &str, hwnd: i64, x: i64, y: i64) {
    invoke(
        auth,
        "cursor_set",
        &[map(vec![
            ("name", text(name)),
            ("hwnd", Value::Int(hwnd)),
            ("x", Value::Int(x)),
            ("y", Value::Int(y)),
            ("client_area", Value::Bool(true)),
        ])],
    )
    .unwrap();
}

#[test]
fn named_cursors_keep_independent_local_positions() {
    let auth = ComputerAuthority::new("http://127.0.0.1:9/computer", vec![]);
    set_cursor(&auth, "rdp", 41, 100, 200);
    set_cursor(&auth, "blender", 42, 10, 20);

    let moved = invoke(
        &auth,
        "cursor_move",
        &[map(vec![
            ("name", text("rdp")),
            ("dx", Value::Float(12.5)),
            ("dy", Value::Int(-8)),
        ])],
    )
    .unwrap();
    assert_eq!(map_number(&moved, "x"), 112.5);
    assert_eq!(map_number(&moved, "y"), 192.0);

    let blender = invoke(
        &auth,
        "cursor_state",
        &[map(vec![("name", text("blender"))])],
    )
    .unwrap();
    assert_eq!(map_number(&blender, "x"), 10.0);
    assert_eq!(map_number(&blender, "y"), 20.0);
}

#[test]
fn narrowed_authority_shares_logical_cursor_registry() {
    let auth = ComputerAuthority::new(
        "http://127.0.0.1:9/computer",
        vec!["computer.click".to_string()],
    );
    set_cursor(&auth, "rdp", 77, 300, 400);
    let narrowed = auth
        .narrow(&map(vec![(
            "scopes",
            Value::List(Rc::new(RefCell::new(vec![text("computer.click")]))),
        )]))
        .unwrap();

    invoke(
        &narrowed,
        "cursor_move",
        &[map(vec![
            ("name", text("rdp")),
            ("dx", Value::Int(5)),
            ("dy", Value::Int(6)),
        ])],
    )
    .unwrap();
    let state = invoke(&auth, "cursor_state", &[map(vec![("name", text("rdp"))])]).unwrap();
    assert_eq!(map_number(&state, "x"), 305.0);
    assert_eq!(map_number(&state, "y"), 406.0);
}

#[test]
fn cursor_click_translates_state_to_existing_bridge_action() {
    let bridge = FakeBridge::new();
    let auth = ComputerAuthority::new(&bridge.endpoint, vec!["computer.click".to_string()]);
    set_cursor(&auth, "rdp", 1234, 640, 480);

    invoke(&auth, "cursor_click", &[map(vec![("name", text("rdp"))])]).unwrap();
    bridge.handle.join().unwrap();

    let request = bridge.body.lock().unwrap();
    assert!(request.contains("\"action\":\"click\""));
    assert!(request.contains("\"hwnd\":1234"));
    assert!(request.contains("\"x\":640"));
    assert!(request.contains("\"y\":480"));
    assert!(request.contains("\"client_area\":true"));
}

#[test]
fn cursor_click_still_requires_bridge_scope() {
    let auth = ComputerAuthority::new("http://127.0.0.1:9/computer", vec![]);
    set_cursor(&auth, "rdp", 9, 1, 2);
    let error = invoke(&auth, "cursor_click", &[map(vec![("name", text("rdp"))])]).unwrap_err();
    assert!(error.contains("computer.click"));
}
