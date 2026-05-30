use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use tetherscript::capability::Authority;
use tetherscript::computer_cap::ComputerAuthority;
use tetherscript::plugin::PluginHost;
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
    fn new(response: &'static str) -> Self {
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
                assert!(n > 0, "bridge request closed before headers");
                data.extend_from_slice(&buf[..n]);
                if String::from_utf8_lossy(&data).contains("\r\n\r\n") {
                    break;
                }
            }
            let mut text = String::from_utf8_lossy(&data).into_owned();
            let split = text.find("\r\n\r\n").unwrap() + 4;
            let content_length = text[..split]
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .unwrap()
                .parse::<usize>()
                .unwrap();
            while text[split..].len() < content_length {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0, "bridge request closed before body");
                data.extend_from_slice(&buf[..n]);
                text = String::from_utf8_lossy(&data).into_owned();
            }
            *body_clone.lock().unwrap() = text[..split + content_length].to_string();
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

fn str_value(text: &str) -> Value {
    Value::Str(Rc::new(text.into()))
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(Rc::new(RefCell::new(
        entries.into_iter().map(|(k, v)| (k.into(), v)).collect(),
    )))
}

fn invoke(auth: &Rc<dyn Authority>, method: &str, args: &[Value]) -> Result<Value, String> {
    auth.invoke(&mut NoopRuntime, method, args)
}

#[test]
fn snapshot_posts_computer_action_envelope_to_bridge() {
    let bridge = FakeBridge::new("{\"ok\":true,\"value\":{\"ready\":true}}");
    let auth = ComputerAuthority::new(&bridge.endpoint, vec!["computer.snapshot".into()]);

    invoke(&auth, "snapshot", &[]).unwrap();
    bridge.handle.join().unwrap();

    assert!(bridge
        .body
        .lock()
        .unwrap()
        .contains("\"action\":\"snapshot\""));
}

#[test]
fn origin_bound_authority_forwards_origin_header() {
    let bridge = FakeBridge::new("{\"ok\":true}");
    let auth = ComputerAuthority::new_origin_bound(
        &bridge.endpoint,
        vec!["computer.snapshot".into()],
        Some("agent://desktop-script".into()),
    );

    invoke(&auth, "snapshot", &[]).unwrap();
    bridge.handle.join().unwrap();

    assert!(bridge
        .body
        .lock()
        .unwrap()
        .contains("X-TetherScript-Origin: agent://desktop-script"));
}

#[test]
fn click_action_requires_click_scope() {
    let auth = ComputerAuthority::new(
        "http://127.0.0.1:9/computer",
        vec!["computer.snapshot".into()],
    );
    let err = invoke(
        &auth,
        "click",
        &[map(vec![("x", Value::Int(1)), ("y", Value::Int(2))])],
    )
    .unwrap_err();
    assert!(err.contains("computer.click"));
}

#[test]
fn raw_action_uses_action_scope_and_merges_params() {
    let bridge = FakeBridge::new("{\"success\":true,\"output\":\"ok\"}");
    let auth = ComputerAuthority::new(&bridge.endpoint, vec!["computer.type".into()]);

    invoke(
        &auth,
        "raw",
        &[
            str_value("type_text"),
            map(vec![("text", str_value("hello"))]),
        ],
    )
    .unwrap();
    bridge.handle.join().unwrap();
    let body = bridge.body.lock().unwrap();
    assert!(body.contains("\"action\":\"type_text\""));
    assert!(body.contains("\"text\":\"hello\""));
}

#[test]
fn plugin_grant_can_script_computer_snapshot() {
    let bridge = FakeBridge::new("{\"ok\":true,\"value\":{\"ready\":true}}");
    let mut host = PluginHost::new();
    host.grant(
        "computer",
        ComputerAuthority::new(&bridge.endpoint, vec!["computer.snapshot".into()]),
    );
    let mut plugin = host
        .load_source("desktop-script", "fn run() { return computer.snapshot() }")
        .unwrap();

    plugin.call("run", &[]).unwrap();
    bridge.handle.join().unwrap();

    assert!(bridge
        .body
        .lock()
        .unwrap()
        .contains("\"action\":\"snapshot\""));
}

#[test]
fn narrow_can_only_remove_scopes() {
    let auth = ComputerAuthority::new(
        "http://127.0.0.1:9/computer",
        vec!["computer.snapshot".into()],
    );
    let params = map(vec![(
        "scopes",
        Value::List(Rc::new(RefCell::new(vec![str_value("computer.click")]))),
    )]);
    let err = match auth.narrow(&params) {
        Ok(_) => panic!("narrow should reject added computer.click scope"),
        Err(err) => err,
    };
    assert!(err.contains("subset"));
}
