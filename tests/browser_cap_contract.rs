use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use tetherscript::browser_cap::BrowserAuthority;
use tetherscript::capability::Authority;
use tetherscript::value::{Runtime, Value};

struct NoopRuntime;

impl Runtime for NoopRuntime {
    fn invoke(&mut self, _callee: &Value, _args: &[Value]) -> Result<Value, String> {
        Ok(Value::Nil)
    }
}

struct FakeBridge {
    endpoint: String,
    request: Arc<Mutex<String>>,
    body: Arc<Mutex<String>>,
    handle: thread::JoinHandle<()>,
}

impl FakeBridge {
    fn new(response: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/browser", listener.local_addr().unwrap());
        let request = Arc::new(Mutex::new(String::new()));
        let body = Arc::new(Mutex::new(String::new()));
        let request_clone = request.clone();
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
            *request_clone.lock().unwrap() = text[..split].to_string();
            *body_clone.lock().unwrap() = text[split..split + content_length].to_string();
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            );
            stream.write_all(reply.as_bytes()).unwrap();
        });
        Self {
            endpoint,
            request,
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
fn goto_posts_browserctl_action_envelope_to_bridge_root() {
    let bridge = FakeBridge::new("{\"ok\":true,\"value\":{\"status\":\"ok\"}}");
    let auth = BrowserAuthority::new(
        &bridge.endpoint,
        vec!["http://app.test".into()],
        vec!["browser.navigate".into()],
    );

    invoke(&auth, "goto", &[str_value("http://app.test/home")]).unwrap();
    bridge.handle.join().unwrap();

    assert!(bridge
        .request
        .lock()
        .unwrap()
        .starts_with("POST /browser HTTP/1.1"));
    let body = bridge.body.lock().unwrap();
    assert!(body.contains("\"action\":\"goto\""));
    assert!(body.contains("\"url\":\"http://app.test/home\""));
}

#[test]
fn raw_actions_are_action_enveloped_and_scope_checked() {
    let bridge = FakeBridge::new("{\"ok\":true,\"value\":[]}");
    let auth = BrowserAuthority::new(
        &bridge.endpoint,
        Vec::new(),
        vec!["browser.inspect.network".into()],
    );

    invoke(
        &auth,
        "raw",
        &[
            str_value("network_log"),
            map(vec![("url_contains", str_value("/api"))]),
        ],
    )
    .unwrap();
    bridge.handle.join().unwrap();

    let body = bridge.body.lock().unwrap();
    assert!(body.contains("\"action\":\"network_log\""));
    assert!(body.contains("\"url_contains\":\"/api\""));
}

#[test]
fn tool_result_output_json_is_normalized_to_value() {
    let bridge = FakeBridge::new("{\"success\":true,\"output\":\"{\\\"title\\\":\\\"ok\\\"}\"}");
    let auth = BrowserAuthority::new(
        &bridge.endpoint,
        Vec::new(),
        vec!["browser.inspect.dom".into()],
    );

    let value = invoke(&auth, "page_snapshot", &[]).unwrap();
    bridge.handle.join().unwrap();

    match value {
        Value::Map(m) => assert_eq!(m.borrow().get("title"), Some(&str_value("ok"))),
        other => panic!("expected map value, got {}", other.type_name()),
    }
}

#[test]
fn missing_scope_denies_before_network_io() {
    let auth = BrowserAuthority::new("http://127.0.0.1:1/browser", Vec::new(), Vec::new());
    let err = invoke(&auth, "goto", &[str_value("http://app.test")]).unwrap_err();
    assert!(err.contains("scope `browser.navigate` not granted"));
}

#[test]
fn origin_denial_happens_before_network_io() {
    let auth = BrowserAuthority::new(
        "http://127.0.0.1:1/browser",
        vec!["http://allowed.test".into()],
        vec!["browser.navigate".into()],
    );
    let err = invoke(&auth, "goto", &[str_value("http://evil.test")]).unwrap_err();
    assert!(err.contains("origin http://evil.test is not granted"));
}
