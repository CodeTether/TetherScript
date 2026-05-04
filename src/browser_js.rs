//! Tiny dependency-free browser JavaScript host bindings.
//!
//! This is a deterministic bridge between the in-tree HTML/CSS browser
//! primitives and the in-tree JavaScript interpreter. It intentionally uses only
//! `std` and the project's own modules. It exposes a small DOM surface:
//! `document`, `window`, `getElementById`, `querySelector`, `querySelectorAll`,
//! `textContent`, `innerText`, `innerHTML`, `children`, `setAttribute`, and
//! `getAttribute`, plus basic element creation and tree mutation APIs. Browser
//! compatibility globals also include `location`, `navigator`, deterministic
//! timers, and in-memory `localStorage`/`sessionStorage` Storage objects.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::browser::{self, Element, Node};
use crate::js::{self, JsEngine, JsValue, NativeFunction};
use crate::value::Value;

const DOM_API_VERSION: &str = "tetherscript-dom-0.3";
const MAX_TIMER_DRAIN: usize = 10_000;

thread_local! {
    static EVENT_REGISTRY: RefCell<HashMap<String, EventEntry>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Copy)]
enum InsertPosition {
    Append,
    Prepend,
}

// NOTE(riley): Index-path-based handles can shift when siblings are prepended/removed.
// A future iteration should give each node a stable ID or store Rc<RefCell<Node>> per node
// so that existing handles remain valid after mutations.
#[derive(Clone)]
struct DomHandle {
    root: Rc<RefCell<Node>>,
    path: Vec<usize>,
}

#[derive(Clone, Default)]
struct EventEntry {
    listeners: HashMap<String, Vec<JsValue>>,
    handlers: HashMap<String, JsValue>,
}

#[derive(Clone)]
struct TimerTask {
    id: u32,
    callback: JsValue,
    args: Vec<JsValue>,
}

#[derive(Default)]
struct TimerQueue {
    next_id: u32,
    tasks: VecDeque<TimerTask>,
}

#[derive(Default)]
struct StorageArea {
    entries: Vec<(String, String)>,
}

pub struct BrowserJsResult {
    pub document: browser::Document,
    pub value: JsValue,
    pub console: Vec<String>,
}

pub fn run_html_scripts(html: &str) -> Result<BrowserJsResult, String> {
    EVENT_REGISTRY.with(|r| r.borrow_mut().clear());
    let root = html_to_root(html);
    let mut engine = JsEngine::new();
    let timers = Rc::new(RefCell::new(TimerQueue::default()));
    let window = install_dom_globals(&mut engine, root.clone(), timers.clone());
    let scripts = collect_inline_scripts(&root.borrow());
    let mut last = JsValue::Undefined;
    for source in scripts {
        if !source.trim().is_empty() {
            last = engine.eval(&source)?;
        }
    }
    drain_timers(timers, window)?;
    Ok(BrowserJsResult {
        document: root_to_document(&root),
        value: last,
        console: engine.console_output(),
    })
}

pub fn eval_with_dom(html: &str, script: &str) -> Result<BrowserJsResult, String> {
    EVENT_REGISTRY.with(|r| r.borrow_mut().clear());
    let root = html_to_root(html);
    let mut engine = JsEngine::new();
    let timers = Rc::new(RefCell::new(TimerQueue::default()));
    let window = install_dom_globals(&mut engine, root.clone(), timers.clone());
    let value = engine.eval(script)?;
    drain_timers(timers, window)?;
    Ok(BrowserJsResult {
        document: root_to_document(&root),
        value,
        console: engine.console_output(),
    })
}

fn html_to_root(html: &str) -> Rc<RefCell<Node>> {
    let document = browser::parse_html(html);
    Rc::new(RefCell::new(Node::Element(Element {
        tag: "#document".into(),
        attrs: HashMap::new(),
        children: document.children,
    })))
}

fn install_dom_globals(
    engine: &mut JsEngine,
    root: Rc<RefCell<Node>>,
    timers: Rc<RefCell<TimerQueue>>,
) -> JsValue {
    let document = node_object(DomHandle {
        root: root.clone(),
        path: Vec::new(),
    });
    engine.set_global("document", document.clone());
    let mut window = HashMap::new();
    window.insert("document".into(), document);
    let location = location_object("http://localhost/");
    let navigator = navigator_object();
    let url_ctor = native("URL", None, |args| {
        let input = args.first().unwrap_or(&JsValue::Undefined).display();
        let base = args
            .get(1)
            .map(JsValue::display)
            .unwrap_or_else(|| "http://localhost/".into());
        Ok(url_object(&resolve_url(&input, &base)))
    });
    let search_params_ctor = native("URLSearchParams", None, |args| {
        Ok(url_search_params_object(
            &args.first().map(JsValue::display).unwrap_or_default(),
        ))
    });
    let local_storage = storage_object("localStorage");
    let session_storage = storage_object("sessionStorage");
    window.insert("location".into(), location.clone());
    window.insert("navigator".into(), navigator.clone());
    window.insert("URL".into(), url_ctor.clone());
    window.insert("URLSearchParams".into(), search_params_ctor.clone());
    window.insert("localStorage".into(), local_storage.clone());
    window.insert("sessionStorage".into(), session_storage.clone());
    install_fetch_bindings(&mut window);
    install_timer_bindings(&mut window, timers);
    let window = JsValue::Object(Rc::new(RefCell::new(window)));
    if let JsValue::Object(obj) = &window {
        obj.borrow_mut().insert("window".into(), window.clone());
        obj.borrow_mut().insert("self".into(), window.clone());
        let borrowed = obj.borrow();
        if let Some(set_timeout) = borrowed.get("setTimeout").cloned() {
            engine.set_global("setTimeout", set_timeout);
        }
        if let Some(clear_timeout) = borrowed.get("clearTimeout").cloned() {
            engine.set_global("clearTimeout", clear_timeout);
        }
        if let Some(fetch) = borrowed.get("fetch").cloned() {
            engine.set_global("fetch", fetch);
        }
        if let Some(headers) = borrowed.get("Headers").cloned() {
            engine.set_global("Headers", headers);
        }
        if let Some(response) = borrowed.get("Response").cloned() {
            engine.set_global("Response", response);
        }
    }
    engine.set_global("window", window.clone());
    engine.set_global("self", window.clone());
    engine.set_global("location", location);
    engine.set_global("navigator", navigator);
    engine.set_global("URL", url_ctor);
    engine.set_global("URLSearchParams", search_params_ctor);
    engine.set_global("localStorage", local_storage);
    engine.set_global("sessionStorage", session_storage);
    if let JsValue::Object(obj) = &window {
        for name in ["fetch", "Headers", "Response"] {
            if let Some(value) = obj.borrow().get(name).cloned() {
                engine.set_global(name, value);
            }
        }
    }
    window
}

fn install_fetch_bindings(window: &mut HashMap<String, JsValue>) {
    let headers_ctor = native("Headers", None, |args| Ok(headers_object(args.first())));
    let response_ctor = native("Response", None, |args| {
        let body = args.first().map(JsValue::display).unwrap_or_default();
        let init = args.get(1);
        let status = object_number(init, "status").unwrap_or(200.0) as u16;
        let status_text =
            object_string(init, "statusText").unwrap_or_else(|| default_status_text(status).into());
        let headers = object_prop(init, "headers").unwrap_or_else(|| headers_object(None));
        Ok(response_object(
            body,
            status,
            status_text,
            headers,
            String::new(),
        ))
    });
    let fetch = native("fetch", None, |args| {
        let url = args.first().map(JsValue::display).unwrap_or_default();
        let mut body = format!("fetch stub: {}", url);
        let mut status = 200;
        if let Some(init) = args.get(1) {
            if let Some(method) = object_string(Some(init), "method") {
                body = format!("fetch stub: {} {}", method.to_ascii_uppercase(), url);
            }
            if let Some(mock_body) = object_string(Some(init), "body") {
                body = mock_body;
            }
            if let Some(mock_status) = object_number(Some(init), "status") {
                status = mock_status as u16;
            }
        }
        let mut headers = vec![("content-type".into(), "text/plain;charset=utf-8".into())];
        if let Some(init_headers) = args
            .get(1)
            .and_then(|init| object_prop(Some(init), "headers"))
        {
            headers.extend(headers_entries(&init_headers));
        }
        Ok(response_object(
            body,
            status,
            default_status_text(status).into(),
            headers_from_entries(headers),
            url,
        ))
    });

    window.insert("Headers".into(), headers_ctor.clone());
    window.insert("Response".into(), response_ctor.clone());
    window.insert("fetch".into(), fetch.clone());
}

fn response_object(
    body: String,
    status: u16,
    status_text: String,
    headers: JsValue,
    url: String,
) -> JsValue {
    let ok = (200..=299).contains(&status);
    let mut obj = HashMap::new();
    obj.insert("body".into(), JsValue::String(body.clone()));
    obj.insert("status".into(), JsValue::Number(status as f64));
    obj.insert("statusText".into(), JsValue::String(status_text));
    obj.insert("ok".into(), JsValue::Bool(ok));
    obj.insert("url".into(), JsValue::String(url));
    obj.insert("headers".into(), headers);
    let text_body = body.clone();
    obj.insert(
        "text".into(),
        native("Response.text", Some(0), move |_| {
            Ok(JsValue::String(text_body.clone()))
        }),
    );
    obj.insert(
        "json".into(),
        native("Response.json", Some(0), move |_| {
            Ok(json_stub_value(&body))
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn headers_object(init: Option<&JsValue>) -> JsValue {
    headers_from_entries(init.into_iter().flat_map(headers_entries).collect())
}

fn headers_from_entries(entries: Vec<(String, String)>) -> JsValue {
    let entries = Rc::new(RefCell::new(normalize_header_entries(entries)));
    let mut obj = HashMap::new();
    obj.insert(
        "get".into(),
        header_method(entries.clone(), "Headers.get", |entries, name, _| {
            entries
                .iter()
                .find(|(k, _)| k == &name)
                .map(|(_, v)| JsValue::String(v.clone()))
                .unwrap_or(JsValue::Null)
        }),
    );
    obj.insert(
        "has".into(),
        header_method(entries.clone(), "Headers.has", |entries, name, _| {
            JsValue::Bool(entries.iter().any(|(k, _)| k == &name))
        }),
    );
    obj.insert(
        "set".into(),
        header_method(entries.clone(), "Headers.set", |entries, name, value| {
            set_header(entries, name, value);
            JsValue::Undefined
        }),
    );
    obj.insert(
        "append".into(),
        header_method(entries.clone(), "Headers.append", |entries, name, value| {
            append_header(entries, name, value);
            JsValue::Undefined
        }),
    );
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn header_method(
    entries: Rc<RefCell<Vec<(String, String)>>>,
    name: &'static str,
    f: fn(&mut Vec<(String, String)>, String, String) -> JsValue,
) -> JsValue {
    native(name, None, move |args| {
        let key = args
            .first()
            .map(JsValue::display)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let value = args.get(1).map(JsValue::display).unwrap_or_default();
        Ok(f(&mut entries.borrow_mut(), key, value))
    })
}

fn normalize_header_entries(entries: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (k, v) in entries {
        append_header(&mut out, k.to_ascii_lowercase(), v);
    }
    out
}
fn set_header(entries: &mut Vec<(String, String)>, name: String, value: String) {
    entries.retain(|(k, _)| k != &name);
    entries.push((name, value));
}
fn append_header(entries: &mut Vec<(String, String)>, name: String, value: String) {
    if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == &name) {
        if !existing.is_empty() {
            existing.push_str(", ");
        }
        existing.push_str(&value);
    } else {
        entries.push((name, value));
    }
}

fn headers_entries(value: &JsValue) -> Vec<(String, String)> {
    match value {
        JsValue::Object(obj) => obj
            .borrow()
            .iter()
            .filter(|(_, v)| {
                !matches!(
                    v,
                    JsValue::Native(_) | JsValue::Function(_) | JsValue::BoundFunction(_)
                )
            })
            .map(|(k, v)| (k.to_ascii_lowercase(), v.display()))
            .collect(),
        _ => Vec::new(),
    }
}
fn object_prop(value: Option<&JsValue>, prop: &str) -> Option<JsValue> {
    match value {
        Some(JsValue::Object(obj)) => obj.borrow().get(prop).cloned(),
        _ => None,
    }
}
fn object_string(value: Option<&JsValue>, prop: &str) -> Option<String> {
    object_prop(value, prop).map(|v| v.display())
}
fn object_number(value: Option<&JsValue>, prop: &str) -> Option<f64> {
    match object_prop(value, prop) {
        Some(JsValue::Number(n)) => Some(n),
        Some(v) => v.display().parse().ok(),
        None => None,
    }
}
fn default_status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "",
    }
}
fn json_stub_value(body: &str) -> JsValue {
    let trimmed = body.trim();
    if trimmed == "null" {
        JsValue::Null
    } else if trimmed == "true" {
        JsValue::Bool(true)
    } else if trimmed == "false" {
        JsValue::Bool(false)
    } else if let Ok(n) = trimmed.parse::<f64>() {
        JsValue::Number(n)
    } else {
        JsValue::String(trimmed.trim_matches('"').into())
    }
}

fn install_timer_bindings(window: &mut HashMap<String, JsValue>, timers: Rc<RefCell<TimerQueue>>) {
    let set_queue = timers.clone();
    window.insert(
        "setTimeout".into(),
        native("setTimeout", None, move |args| {
            let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
            let callback_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            let mut queue = set_queue.borrow_mut();
            queue.next_id = queue.next_id.saturating_add(1).max(1);
            let id = queue.next_id;
            queue.tasks.push_back(TimerTask {
                id,
                callback,
                args: callback_args,
            });
            Ok(JsValue::Number(id as f64))
        }),
    );

    let clear_queue = timers;
    window.insert(
        "clearTimeout".into(),
        native("clearTimeout", None, move |args| {
            let id = args.first().map(timer_id).unwrap_or(0);
            clear_queue.borrow_mut().tasks.retain(|task| task.id != id);
            Ok(JsValue::Undefined)
        }),
    );
}

fn drain_timers(timers: Rc<RefCell<TimerQueue>>, window: JsValue) -> Result<(), String> {
    let mut drained = 0;
    loop {
        let task = { timers.borrow_mut().tasks.pop_front() };
        let Some(task) = task else {
            break;
        };
        drained += 1;
        if drained > MAX_TIMER_DRAIN {
            return Err(format!(
                "setTimeout: exceeded deterministic drain limit of {} callbacks",
                MAX_TIMER_DRAIN
            ));
        }
        js::call_function_with_this(task.callback, window.clone(), &task.args)?;
    }
    Ok(())
}

fn timer_id(value: &JsValue) -> u32 {
    match value {
        JsValue::Number(n) if n.is_finite() && *n > 0.0 => *n as u32,
        other => other.display().parse().unwrap_or(0),
    }
}

fn location_object(href: &str) -> JsValue {
    let object = Rc::new(RefCell::new(parse_location(&normalize_url(href))));
    {
        let object = object.clone();
        let object_for_closure = object.clone();
        object.borrow_mut().insert(
            "assign".into(),
            native("location.assign", Some(1), move |args| {
                let current = object_for_closure
                    .borrow()
                    .get("href")
                    .map(JsValue::display)
                    .unwrap_or_else(|| "http://localhost/".into());
                let next = resolve_url(
                    &args.first().unwrap_or(&JsValue::Undefined).display(),
                    &current,
                );
                update_url_like_object(&object_for_closure, &next);
                Ok(JsValue::Undefined)
            }),
        );
    }
    {
        let object = object.clone();
        let object_for_closure = object.clone();
        object.borrow_mut().insert(
            "replace".into(),
            native("location.replace", Some(1), move |args| {
                let current = object_for_closure
                    .borrow()
                    .get("href")
                    .map(JsValue::display)
                    .unwrap_or_else(|| "http://localhost/".into());
                let next = resolve_url(
                    &args.first().unwrap_or(&JsValue::Undefined).display(),
                    &current,
                );
                update_url_like_object(&object_for_closure, &next);
                Ok(JsValue::Undefined)
            }),
        );
    }
    JsValue::Object(object)
}

fn url_object(href: &str) -> JsValue {
    let object = Rc::new(RefCell::new(parse_location(&normalize_url(href))));
    let search = object
        .borrow()
        .get("search")
        .map(JsValue::display)
        .unwrap_or_default();
    object
        .borrow_mut()
        .insert("searchParams".into(), url_search_params_object(&search));
    {
        let object = object.clone();
        let object_for_closure = object.clone();
        object.borrow_mut().insert(
            "toString".into(),
            native("URL.toString", Some(0), move |_| {
                Ok(JsValue::String(
                    object_for_closure
                        .borrow()
                        .get("href")
                        .map(JsValue::display)
                        .unwrap_or_default(),
                ))
            }),
        );
    }
    object.borrow_mut().insert("__set:href".into(), {
        let object = object.clone();
        native("set_URL_href", Some(1), move |args| {
            update_url_like_object(
                &object,
                &normalize_url(&args.first().unwrap_or(&JsValue::Undefined).display()),
            );
            Ok(JsValue::Undefined)
        })
    });
    JsValue::Object(object)
}

fn update_url_like_object(object: &Rc<RefCell<HashMap<String, JsValue>>>, href: &str) {
    let methods = {
        let obj = object.borrow();
        ["assign", "replace", "toString", "__set:href"]
            .into_iter()
            .filter_map(|k| obj.get(k).cloned().map(|v| (k.to_string(), v)))
            .collect::<Vec<_>>()
    };
    let mut parsed = parse_location(&normalize_url(href));
    let search = parsed
        .get("search")
        .map(JsValue::display)
        .unwrap_or_default();
    parsed.insert("searchParams".into(), url_search_params_object(&search));
    for (k, v) in methods {
        parsed.insert(k, v);
    }
    *object.borrow_mut() = parsed;
}

fn navigator_object() -> JsValue {
    let mut obj = HashMap::new();
    obj.insert(
        "userAgent".into(),
        JsValue::String("TetherScript/0.1 BrowserCompat".into()),
    );
    obj.insert("language".into(), JsValue::String("en-US".into()));
    obj.insert(
        "languages".into(),
        JsValue::Array(Rc::new(RefCell::new(vec![
            JsValue::String("en-US".into()),
            JsValue::String("en".into()),
        ]))),
    );
    obj.insert(
        "platform".into(),
        JsValue::String(std::env::consts::OS.into()),
    );
    obj.insert("cookieEnabled".into(), JsValue::Bool(false));
    obj.insert("onLine".into(), JsValue::Bool(true));
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn storage_object(label: &'static str) -> JsValue {
    let area = Rc::new(RefCell::new(StorageArea::default()));
    let object = Rc::new(RefCell::new(HashMap::new()));

    object
        .borrow_mut()
        .insert("length".into(), JsValue::Number(0.0));

    {
        let area = area.clone();
        object.borrow_mut().insert(
            "getItem".into(),
            native(&format!("{}.getItem", label), Some(1), move |args| {
                let key = args.first().unwrap_or(&JsValue::Undefined).display();
                Ok(area
                    .borrow()
                    .get(&key)
                    .map(JsValue::String)
                    .unwrap_or(JsValue::Null))
            }),
        );
    }

    {
        let area = area.clone();
        let object = object.clone();
        let object_for_closure = object.clone();
        let set_item = native(&format!("{}.setItem", label), Some(2), move |args| {
            let key = args.first().unwrap_or(&JsValue::Undefined).display();
            let value = args.get(1).unwrap_or(&JsValue::Undefined).display();
            area.borrow_mut().set(key, value);
            sync_storage_length(&object_for_closure, &area);
            Ok(JsValue::Undefined)
        });
        object.borrow_mut().insert("setItem".into(), set_item);
    }

    {
        let area = area.clone();
        let object = object.clone();
        let object_for_closure = object.clone();
        let remove_item = native(&format!("{}.removeItem", label), Some(1), move |args| {
            let key = args.first().unwrap_or(&JsValue::Undefined).display();
            area.borrow_mut().remove(&key);
            sync_storage_length(&object_for_closure, &area);
            Ok(JsValue::Undefined)
        });
        object.borrow_mut().insert("removeItem".into(), remove_item);
    }

    {
        let area = area.clone();
        let object = object.clone();
        let object_for_closure = object.clone();
        let clear = native(&format!("{}.clear", label), Some(0), move |_| {
            area.borrow_mut().clear();
            sync_storage_length(&object_for_closure, &area);
            Ok(JsValue::Undefined)
        });
        object.borrow_mut().insert("clear".into(), clear);
    }

    {
        let area = area.clone();
        object.borrow_mut().insert(
            "key".into(),
            native(&format!("{}.key", label), Some(1), move |args| {
                let index = args.first().map(storage_index).unwrap_or(usize::MAX);
                Ok(area
                    .borrow()
                    .key(index)
                    .map(JsValue::String)
                    .unwrap_or(JsValue::Null))
            }),
        );
    }

    JsValue::Object(object)
}

fn sync_storage_length(
    object: &Rc<RefCell<HashMap<String, JsValue>>>,
    area: &Rc<RefCell<StorageArea>>,
) {
    object
        .borrow_mut()
        .insert("length".into(), JsValue::Number(area.borrow().len() as f64));
}

fn storage_index(value: &JsValue) -> usize {
    match value {
        JsValue::Number(n) if n.is_finite() && *n >= 0.0 => *n as usize,
        other => other.display().parse().unwrap_or(usize::MAX),
    }
}

impl StorageArea {
    fn get(&self, key: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|(existing, _)| existing == key)
            .map(|(_, value)| value.clone())
    }
    fn set(&mut self, key: String, value: String) {
        if let Some((_, existing)) = self
            .entries
            .iter_mut()
            .find(|(existing, _)| existing == &key)
        {
            *existing = value;
        } else {
            self.entries.push((key, value));
        }
    }
    fn remove(&mut self, key: &str) {
        self.entries.retain(|(existing, _)| existing != key);
    }
    fn clear(&mut self) {
        self.entries.clear();
    }
    fn key(&self, index: usize) -> Option<String> {
        self.entries.get(index).map(|(key, _)| key.clone())
    }
    fn len(&self) -> usize {
        self.entries.len()
    }
}

fn node_object(handle: DomHandle) -> JsValue {
    let node = handle.node().unwrap_or(Node::Text(String::new()));
    let mut obj = HashMap::new();
    obj.insert(
        "nodeType".into(),
        JsValue::Number(if matches!(node, Node::Text(_)) {
            3.0
        } else if node_name(&node) == "#document" {
            9.0
        } else {
            1.0
        }),
    );
    obj.insert("nodeName".into(), JsValue::String(node_name(&node)));
    obj.insert(
        "tagName".into(),
        JsValue::String(node_name(&node).to_ascii_uppercase()),
    );
    obj.insert(
        "textContent".into(),
        JsValue::String(text_content_raw(&node)),
    );
    obj.insert(
        "innerText".into(),
        JsValue::String(browser::text_content(&node)),
    );
    obj.insert("innerHTML".into(), JsValue::String(inner_html(&node)));
    obj.insert("children".into(), children_array(&handle, &node));
    obj.insert(
        "childElementCount".into(),
        JsValue::Number(child_element_count(&node) as f64),
    );
    obj.insert(
        "parentNode".into(),
        handle
            .parent()
            .map(shallow_node_object)
            .unwrap_or(JsValue::Null),
    );

    if let Node::Element(el) = &node {
        obj.insert(
            "id".into(),
            JsValue::String(el.attrs.get("id").cloned().unwrap_or_default()),
        );
        obj.insert(
            "className".into(),
            JsValue::String(el.attrs.get("class").cloned().unwrap_or_default()),
        );
        obj.insert(
            "value".into(),
            JsValue::String(el.attrs.get("value").cloned().unwrap_or_default()),
        );
        obj.insert(
            "checked".into(),
            JsValue::Bool(el.attrs.contains_key("checked")),
        );
    }

    install_property_setters(&mut obj, &handle);

    let h = handle.clone();
    obj.insert(
        "createElement".into(),
        native("createElement", Some(1), move |args| {
            let tag = args
                .first()
                .unwrap_or(&JsValue::Undefined)
                .display()
                .to_ascii_lowercase();
            let path = h.append_child(
                Node::Element(Element {
                    tag,
                    attrs: HashMap::new(),
                    children: Vec::new(),
                }),
                InsertPosition::Append,
            );
            Ok(node_object(DomHandle {
                root: h.root.clone(),
                path,
            }))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "createTextNode".into(),
        native("createTextNode", Some(1), move |args| {
            let text = args.first().unwrap_or(&JsValue::Undefined).display();
            let path = h.append_child(Node::Text(text), InsertPosition::Append);
            Ok(node_object(DomHandle {
                root: h.root.clone(),
                path,
            }))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "appendChild".into(),
        native("appendChild", Some(1), move |args| {
            let child = js_value_to_node(args.first().unwrap_or(&JsValue::Undefined));
            let path = h.append_child(child, InsertPosition::Append);
            Ok(node_object(DomHandle {
                root: h.root.clone(),
                path,
            }))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "getAttribute".into(),
        native("getAttribute", Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(match h.node() {
                Some(Node::Element(el)) => el
                    .attrs
                    .get(&name)
                    .map(|s| JsValue::String(s.clone()))
                    .unwrap_or(JsValue::Null),
                _ => JsValue::Null,
            })
        }),
    );

    let h = handle.clone();
    obj.insert(
        "setAttribute".into(),
        native("setAttribute", Some(2), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            let value = args.get(1).unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    el.attrs.insert(name, value);
                }
            });
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "removeAttribute".into(),
        native("removeAttribute", Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    el.attrs.remove(&name);
                }
            });
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "hasAttribute".into(),
        native("hasAttribute", Some(1), move |args| {
            let name = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(JsValue::Bool(
                matches!(h.node(), Some(Node::Element(el)) if el.attrs.contains_key(&name)),
            ))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "remove".into(),
        native("remove", Some(0), move |_| {
            h.remove_self();
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "prepend".into(),
        native("prepend", None, move |args| {
            for arg in args.iter().rev() {
                h.append_child(js_value_to_node(arg), InsertPosition::Prepend);
            }
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "append".into(),
        native("append", None, move |args| {
            for arg in args {
                h.append_child(js_value_to_node(arg), InsertPosition::Append);
            }
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "getElementById".into(),
        native("getElementById", Some(1), move |args| {
            let id = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(find_by_id(&h.root, &id)
                .map(|path| {
                    node_object(DomHandle {
                        root: h.root.clone(),
                        path,
                    })
                })
                .unwrap_or(JsValue::Null))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "querySelector".into(),
        native("querySelector", Some(1), move |args| {
            let selector = args.first().unwrap_or(&JsValue::Undefined).display();
            Ok(find_by_selector(&h.root, &selector)
                .map(|path| {
                    node_object(DomHandle {
                        root: h.root.clone(),
                        path,
                    })
                })
                .unwrap_or(JsValue::Null))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "querySelectorAll".into(),
        native("querySelectorAll", Some(1), move |args| {
            let selector = args.first().unwrap_or(&JsValue::Undefined).display();
            let nodes = all_by_selector(&h.root, &selector)
                .into_iter()
                .map(|path| {
                    node_object(DomHandle {
                        root: h.root.clone(),
                        path,
                    })
                })
                .collect();
            Ok(JsValue::Array(Rc::new(RefCell::new(nodes))))
        }),
    );

    let h = handle.clone();
    obj.insert(
        "addEventListener".into(),
        native("addEventListener", Some(2), move |args| {
            let event_type = args.first().unwrap_or(&JsValue::Undefined).display();
            let listener = args.get(1).cloned().unwrap_or(JsValue::Undefined);
            h.add_event_listener(&event_type, listener);
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "removeEventListener".into(),
        native("removeEventListener", Some(2), move |args| {
            let event_type = args.first().unwrap_or(&JsValue::Undefined).display();
            let listener = args.get(1).cloned().unwrap_or(JsValue::Undefined);
            h.remove_event_listener(&event_type, &listener);
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "dispatchEvent".into(),
        native("dispatchEvent", Some(1), move |args| {
            let event = args.first().cloned().unwrap_or(JsValue::Undefined);
            h.dispatch_event(event)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "click".into(),
        native("click", Some(0), move |_| {
            h.dispatch_event(JsValue::String("click".into()))
        }),
    );

    obj.insert(
        "__domApiVersion".into(),
        JsValue::String(DOM_API_VERSION.into()),
    );

    JsValue::Object(Rc::new(RefCell::new(obj)))
}

impl DomHandle {
    fn node(&self) -> Option<Node> {
        get_node(&self.root.borrow(), &self.path).cloned()
    }

    fn with_node_mut(&self, f: impl FnOnce(&mut Node)) {
        if let Some(node) = get_node_mut(&mut self.root.borrow_mut(), &self.path) {
            f(node);
        }
    }

    fn parent(&self) -> Option<DomHandle> {
        let (_, parent_path) = self.path.split_last()?;
        Some(DomHandle {
            root: self.root.clone(),
            path: parent_path.to_vec(),
        })
    }

    fn bubble_path(&self) -> Vec<DomHandle> {
        let mut path = self.path.clone();
        let mut out = Vec::new();
        loop {
            out.push(DomHandle {
                root: self.root.clone(),
                path: path.clone(),
            });
            if path.pop().is_none() {
                break;
            }
        }
        out
    }

    fn append_child(&self, child: Node, position: InsertPosition) -> Vec<usize> {
        let mut root = self.root.borrow_mut();
        let parent = get_node_mut(&mut root, &self.path);
        let Some(Node::Element(el)) = parent else {
            return self.path.clone();
        };
        let index = match position {
            InsertPosition::Append => {
                el.children.push(child);
                el.children.len() - 1
            }
            InsertPosition::Prepend => {
                el.children.insert(0, child);
                0
            }
        };
        let mut path = self.path.clone();
        path.push(index);
        path
    }

    fn remove_self(&self) -> bool {
        let Some((&index, parent_path)) = self.path.split_last() else {
            return false;
        };
        let mut root = self.root.borrow_mut();
        let Some(Node::Element(parent)) = get_node_mut(&mut root, parent_path) else {
            return false;
        };
        if index < parent.children.len() {
            parent.children.remove(index);
            true
        } else {
            false
        }
    }

    fn event_key(&self) -> String {
        format!("{:p}:{:?}", Rc::as_ptr(&self.root), self.path)
    }

    fn add_event_listener(&self, event_type: &str, listener: JsValue) {
        EVENT_REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .entry(self.event_key())
                .or_default()
                .listeners
                .entry(event_type.into())
                .or_default()
                .push(listener);
        });
    }

    fn remove_event_listener(&self, event_type: &str, listener: &JsValue) {
        EVENT_REGISTRY.with(|registry| {
            if let Some(entry) = registry.borrow_mut().get_mut(&self.event_key()) {
                if let Some(list) = entry.listeners.get_mut(event_type) {
                    list.retain(|item| item != listener);
                }
            }
        });
    }

    fn set_handler(&self, name: &str, handler: JsValue) {
        EVENT_REGISTRY.with(|registry| {
            registry
                .borrow_mut()
                .entry(self.event_key())
                .or_default()
                .handlers
                .insert(name.into(), handler);
        });
    }

    fn dispatch_event(&self, event: JsValue) -> Result<JsValue, String> {
        let event_type = event_type(&event).unwrap_or_else(|| "event".into());
        let target = node_object(self.clone());
        let event = normalize_event(event, &event_type, target, node_object(self.clone()));
        for current in self.bubble_path() {
            let current_target = node_object(current.clone());
            set_event_current_target(&event, current_target.clone());
            for listener in current.listeners(&event_type) {
                call_dom_listener(listener, current_target.clone(), event.clone())?;
            }
            if let Some(handler) = current.handler(&format!("on{}", event_type)) {
                if call_dom_listener(handler, current_target, event.clone())?
                    == JsValue::Bool(false)
                {
                    set_event_default_prevented(&event);
                }
            }
        }
        Ok(JsValue::Bool(!event_default_prevented(&event)))
    }

    fn listeners(&self, event_type: &str) -> Vec<JsValue> {
        EVENT_REGISTRY.with(|registry| {
            registry
                .borrow()
                .get(&self.event_key())
                .and_then(|entry| entry.listeners.get(event_type).cloned())
                .unwrap_or_default()
        })
    }

    fn handler(&self, name: &str) -> Option<JsValue> {
        EVENT_REGISTRY.with(|registry| {
            registry
                .borrow()
                .get(&self.event_key())
                .and_then(|entry| entry.handlers.get(name).cloned())
        })
    }
}

fn install_property_setters(obj: &mut HashMap<String, JsValue>, handle: &DomHandle) {
    for prop in ["textContent", "innerText"] {
        let h = handle.clone();
        obj.insert(
            format!("__set:{}", prop),
            native(&format!("set_{}", prop), Some(1), move |args| {
                let text = args.first().unwrap_or(&JsValue::Undefined).display();
                h.with_node_mut(|node| match node {
                    Node::Text(existing) => *existing = text,
                    Node::Element(el) => el.children = vec![Node::Text(text)],
                });
                Ok(JsValue::Undefined)
            }),
        );

        for prop in ["onclick", "oninput", "onchange", "onsubmit"] {
            let h = handle.clone();
            obj.insert(
                format!("__set:{}", prop),
                native(&format!("set_{}", prop), Some(1), move |args| {
                    h.set_handler(prop, args.first().cloned().unwrap_or(JsValue::Undefined));
                    Ok(JsValue::Undefined)
                }),
            );
        }
    }

    let h = handle.clone();
    obj.insert(
        "__set:id".into(),
        native("set_id", Some(1), move |args| {
            let value = args.first().unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    el.attrs.insert("id".into(), value);
                }
            });
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "__set:className".into(),
        native("set_className", Some(1), move |args| {
            let value = args.first().unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    el.attrs.insert("class".into(), value);
                }
            });
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "__set:value".into(),
        native("set_value", Some(1), move |args| {
            let value = args.first().unwrap_or(&JsValue::Undefined).display();
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    el.attrs.insert("value".into(), value);
                }
            });
            Ok(JsValue::Undefined)
        }),
    );

    let h = handle.clone();
    obj.insert(
        "__set:checked".into(),
        native("set_checked", Some(1), move |args| {
            let checked = args.first().is_some_and(JsValue::truthy);
            h.with_node_mut(|node| {
                if let Node::Element(el) = node {
                    if checked {
                        el.attrs.insert("checked".into(), "checked".into());
                    } else {
                        el.attrs.remove("checked");
                    }
                }
            });
            Ok(JsValue::Undefined)
        }),
    );
}

fn shallow_node_object(handle: DomHandle) -> JsValue {
    let node = handle.node().unwrap_or(Node::Text(String::new()));
    let mut obj = HashMap::new();
    obj.insert(
        "nodeType".into(),
        JsValue::Number(if matches!(node, Node::Text(_)) {
            3.0
        } else if node_name(&node) == "#document" {
            9.0
        } else {
            1.0
        }),
    );
    obj.insert("nodeName".into(), JsValue::String(node_name(&node)));
    obj.insert(
        "tagName".into(),
        JsValue::String(node_name(&node).to_ascii_uppercase()),
    );
    obj.insert(
        "textContent".into(),
        JsValue::String(text_content_raw(&node)),
    );
    if let Node::Element(el) = &node {
        obj.insert(
            "id".into(),
            JsValue::String(el.attrs.get("id").cloned().unwrap_or_default()),
        );
        obj.insert(
            "className".into(),
            JsValue::String(el.attrs.get("class").cloned().unwrap_or_default()),
        );
        obj.insert(
            "value".into(),
            JsValue::String(el.attrs.get("value").cloned().unwrap_or_default()),
        );
        obj.insert(
            "checked".into(),
            JsValue::Bool(el.attrs.contains_key("checked")),
        );
    }
    install_property_setters(&mut obj, &handle);
    JsValue::Object(Rc::new(RefCell::new(obj)))
}

fn js_value_to_node(value: &JsValue) -> Node {
    match value {
        JsValue::Object(obj) => {
            let obj = obj.borrow();
            if let Some(JsValue::String(tag)) = obj.get("nodeName") {
                if tag != "#text" && tag != "#document" {
                    let mut attrs = HashMap::new();
                    if let Some(JsValue::String(id)) = obj.get("id") {
                        if !id.is_empty() {
                            attrs.insert("id".into(), id.clone());
                        }
                    }
                    if let Some(JsValue::String(class_name)) = obj.get("className") {
                        if !class_name.is_empty() {
                            attrs.insert("class".into(), class_name.clone());
                        }
                    }
                    let text = obj
                        .get("textContent")
                        .map(JsValue::display)
                        .unwrap_or_default();
                    return Node::Element(Element {
                        tag: tag.to_ascii_lowercase(),
                        attrs,
                        children: if text.is_empty() {
                            Vec::new()
                        } else {
                            vec![Node::Text(text)]
                        },
                    });
                }
            }
            Node::Text(value.display())
        }
        _ => Node::Text(value.display()),
    }
}

fn get_node<'a>(node: &'a Node, path: &[usize]) -> Option<&'a Node> {
    if path.is_empty() {
        return Some(node);
    }
    match node {
        Node::Element(el) => el
            .children
            .get(path[0])
            .and_then(|child| get_node(child, &path[1..])),
        Node::Text(_) => None,
    }
}

fn get_node_mut<'a>(node: &'a mut Node, path: &[usize]) -> Option<&'a mut Node> {
    if path.is_empty() {
        return Some(node);
    }
    match node {
        Node::Element(el) => el
            .children
            .get_mut(path[0])
            .and_then(|child| get_node_mut(child, &path[1..])),
        Node::Text(_) => None,
    }
}

fn call_dom_listener(
    listener: JsValue,
    this_value: JsValue,
    event: JsValue,
) -> Result<JsValue, String> {
    js::call_function_with_this(listener, this_value, std::slice::from_ref(&event))
}

fn event_type(event: &JsValue) -> Option<String> {
    match event {
        JsValue::Object(obj) => obj.borrow().get("type").map(JsValue::display),
        JsValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn normalize_event(
    event: JsValue,
    event_type: &str,
    target: JsValue,
    current_target: JsValue,
) -> JsValue {
    let mut map = match event {
        JsValue::Object(obj) => obj.borrow().clone(),
        _ => HashMap::new(),
    };
    map.insert("type".into(), JsValue::String(event_type.into()));
    map.insert("target".into(), target);
    map.insert("currentTarget".into(), current_target);
    map.entry("defaultPrevented".into())
        .or_insert(JsValue::Bool(false));
    let event_ref = Rc::new(RefCell::new(map));
    let event_for_prevent = event_ref.clone();
    event_ref.borrow_mut().insert(
        "preventDefault".into(),
        native("preventDefault", Some(0), move |_| {
            event_for_prevent
                .borrow_mut()
                .insert("defaultPrevented".into(), JsValue::Bool(true));
            Ok(JsValue::Undefined)
        }),
    );
    JsValue::Object(event_ref)
}

fn set_event_current_target(event: &JsValue, current_target: JsValue) {
    if let JsValue::Object(obj) = event {
        obj.borrow_mut()
            .insert("currentTarget".into(), current_target);
    }
}

fn set_event_default_prevented(event: &JsValue) {
    if let JsValue::Object(obj) = event {
        obj.borrow_mut()
            .insert("defaultPrevented".into(), JsValue::Bool(true));
    }
}

fn event_default_prevented(event: &JsValue) -> bool {
    matches!(event, JsValue::Object(obj) if obj.borrow().get("defaultPrevented").is_some_and(JsValue::truthy))
}

fn normalize_url(href: &str) -> String {
    if href.contains("://") {
        href.to_string()
    } else {
        resolve_url(href, "http://localhost/")
    }
}

fn resolve_url(input: &str, base: &str) -> String {
    if input.contains("://") {
        return input.to_string();
    }
    let base = normalize_url(base);
    let base_parts = parse_location(&base);
    let origin = base_parts
        .get("origin")
        .map(JsValue::display)
        .unwrap_or_else(|| "http://localhost".into());
    if input.starts_with("//") {
        let protocol = base_parts
            .get("protocol")
            .map(JsValue::display)
            .unwrap_or_else(|| "http:".into());
        return format!("{}{}", protocol, input);
    }
    if input.starts_with('#') {
        let no_hash = base.split('#').next().unwrap_or(&base);
        return format!("{}{}", no_hash, input);
    }
    if input.starts_with('?') {
        let no_query = base
            .split('?')
            .next()
            .unwrap_or(&base)
            .split('#')
            .next()
            .unwrap_or(&base);
        return format!("{}{}", no_query, input);
    }
    if input.starts_with('/') {
        return format!("{}{}", origin, input);
    }
    let path = base_parts
        .get("pathname")
        .map(JsValue::display)
        .unwrap_or_else(|| "/".into());
    let dir = path
        .rsplit_once('/')
        .map(|(d, _)| format!("{}/", d))
        .unwrap_or_else(|| "/".into());
    format!("{}{}{}", origin, dir, input)
}

fn url_search_params_object(init: &str) -> JsValue {
    let entries = Rc::new(RefCell::new(parse_search_params(init)));
    let object = Rc::new(RefCell::new(HashMap::new()));
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "get".into(),
            native("URLSearchParams.get", Some(1), move |args| {
                let key = args.first().unwrap_or(&JsValue::Undefined).display();
                Ok(entries
                    .borrow()
                    .iter()
                    .find(|(k, _)| k == &key)
                    .map(|(_, v)| JsValue::String(v.clone()))
                    .unwrap_or(JsValue::Null))
            }),
        );
    }
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "set".into(),
            native("URLSearchParams.set", Some(2), move |args| {
                let key = args.first().unwrap_or(&JsValue::Undefined).display();
                let value = args.get(1).unwrap_or(&JsValue::Undefined).display();
                let mut entries = entries.borrow_mut();
                entries.retain(|(k, _)| k != &key);
                entries.push((key, value));
                Ok(JsValue::Undefined)
            }),
        );
    }
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "append".into(),
            native("URLSearchParams.append", Some(2), move |args| {
                entries.borrow_mut().push((
                    args.first().unwrap_or(&JsValue::Undefined).display(),
                    args.get(1).unwrap_or(&JsValue::Undefined).display(),
                ));
                Ok(JsValue::Undefined)
            }),
        );
    }
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "has".into(),
            native("URLSearchParams.has", Some(1), move |args| {
                let key = args.first().unwrap_or(&JsValue::Undefined).display();
                Ok(JsValue::Bool(
                    entries.borrow().iter().any(|(k, _)| k == &key),
                ))
            }),
        );
    }
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "delete".into(),
            native("URLSearchParams.delete", Some(1), move |args| {
                let key = args.first().unwrap_or(&JsValue::Undefined).display();
                entries.borrow_mut().retain(|(k, _)| k != &key);
                Ok(JsValue::Undefined)
            }),
        );
    }
    {
        let entries = entries.clone();
        object.borrow_mut().insert(
            "toString".into(),
            native("URLSearchParams.toString", Some(0), move |_| {
                Ok(JsValue::String(serialize_search_params(&entries.borrow())))
            }),
        );
    }
    JsValue::Object(object)
}

fn parse_search_params(init: &str) -> Vec<(String, String)> {
    let query = init.strip_prefix('?').unwrap_or(init);
    if query.is_empty() {
        return Vec::new();
    }
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            (decode_query_component(key), decode_query_component(value))
        })
        .collect()
}

fn serialize_search_params(entries: &[(String, String)]) -> String {
    entries
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                encode_query_component(k),
                encode_query_component(v)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn decode_query_component(input: &str) -> String {
    input.replace('+', " ")
}
fn encode_query_component(input: &str) -> String {
    input.replace(' ', "+")
}

fn parse_location(href: &str) -> HashMap<String, JsValue> {
    let mut protocol = "http:".to_string();
    let mut host = "localhost".to_string();
    let mut pathname = "/".to_string();
    let mut search = String::new();
    let mut hash = String::new();
    if let Some((p, rest)) = href.split_once("://") {
        protocol = format!("{}:", p);
        let (before_hash, h) = rest.split_once('#').map_or((rest, ""), |(a, b)| (a, b));
        hash = if h.is_empty() {
            String::new()
        } else {
            format!("#{}", h)
        };
        let (before_query, q) = before_hash
            .split_once('?')
            .map_or((before_hash, ""), |(a, b)| (a, b));
        search = if q.is_empty() {
            String::new()
        } else {
            format!("?{}", q)
        };
        if let Some((parsed_host, path)) = before_query.split_once('/') {
            host = parsed_host.into();
            pathname = format!("/{}", path);
        } else {
            host = before_query.into();
            pathname = "/".into();
        }
    }
    let origin = format!("{}//{}", protocol, host);
    let mut obj = HashMap::new();
    for (k, v) in [
        ("href", href.to_string()),
        ("protocol", protocol),
        ("host", host.clone()),
        ("hostname", host),
        ("pathname", pathname),
        ("search", search),
        ("hash", hash),
        ("origin", origin),
    ] {
        obj.insert(k.into(), JsValue::String(v));
    }
    obj
}

fn native(
    name: &str,
    arity: Option<usize>,
    func: impl Fn(&[JsValue]) -> Result<JsValue, String> + 'static,
) -> JsValue {
    JsValue::Native(Rc::new(NativeFunction::new(name, arity, func)))
}

fn root_to_document(root: &Rc<RefCell<Node>>) -> browser::Document {
    match &*root.borrow() {
        Node::Element(el) if el.tag == "#document" => browser::Document {
            children: el.children.clone(),
        },
        node => browser::Document {
            children: vec![node.clone()],
        },
    }
}

fn collect_inline_scripts(node: &Node) -> Vec<String> {
    let mut out = Vec::new();
    collect_scripts(node, &mut out);
    out
}

fn collect_scripts(node: &Node, out: &mut Vec<String>) {
    if let Node::Element(el) = node {
        if el.tag.eq_ignore_ascii_case("script") && !el.attrs.contains_key("src") {
            out.push(
                el.children
                    .iter()
                    .map(text_content_raw)
                    .collect::<Vec<_>>()
                    .join(""),
            );
        }
        for child in &el.children {
            collect_scripts(child, out);
        }
    }
}

fn find_by_id(root: &Rc<RefCell<Node>>, id: &str) -> Option<Vec<usize>> {
    find_path(
        &root.borrow(),
        &mut Vec::new(),
        &|node| matches!(node, Node::Element(el) if el.attrs.get("id").map(|s| s.as_str()) == Some(id)),
    )
}

fn find_by_selector(root: &Rc<RefCell<Node>>, selector: &str) -> Option<Vec<usize>> {
    find_path(&root.borrow(), &mut Vec::new(), &|node| {
        node_matches_simple_selector(node, selector)
    })
}

fn all_by_selector(root: &Rc<RefCell<Node>>, selector: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    collect_paths(
        &root.borrow(),
        &mut Vec::new(),
        &|node| node_matches_simple_selector(node, selector),
        &mut out,
    );
    out
}

fn find_path(
    node: &Node,
    path: &mut Vec<usize>,
    pred: &impl Fn(&Node) -> bool,
) -> Option<Vec<usize>> {
    if pred(node) {
        return Some(path.clone());
    }
    if let Node::Element(el) = node {
        for (index, child) in el.children.iter().enumerate() {
            path.push(index);
            if let Some(found) = find_path(child, path, pred) {
                return Some(found);
            }
            path.pop();
        }
    }
    None
}

fn collect_paths(
    node: &Node,
    path: &mut Vec<usize>,
    pred: &impl Fn(&Node) -> bool,
    out: &mut Vec<Vec<usize>>,
) {
    if pred(node) {
        out.push(path.clone());
    }
    if let Node::Element(el) = node {
        for (index, child) in el.children.iter().enumerate() {
            path.push(index);
            collect_paths(child, path, pred, out);
            path.pop();
        }
    }
}

fn node_matches_simple_selector(node: &Node, selector: &str) -> bool {
    let Node::Element(el) = node else {
        return false;
    };
    let selector = selector.trim();
    if selector.is_empty() {
        return false;
    }
    if let Some(id) = selector.strip_prefix('#') {
        return el.attrs.get("id").map(|s| s == id).unwrap_or(false);
    }
    if let Some(class) = selector.strip_prefix('.') {
        return el
            .attrs
            .get("class")
            .map(|s| s.split_whitespace().any(|c| c == class))
            .unwrap_or(false);
    }
    el.tag.eq_ignore_ascii_case(selector)
}

fn node_name(node: &Node) -> String {
    match node {
        Node::Element(el) => el.tag.clone(),
        Node::Text(_) => "#text".into(),
    }
}

fn children_array(handle: &DomHandle, node: &Node) -> JsValue {
    let len = match node {
        Node::Element(el) => el.children.len(),
        Node::Text(_) => 0,
    };
    let children = (0..len)
        .map(|index| {
            let mut path = handle.path.clone();
            path.push(index);
            node_object(DomHandle {
                root: handle.root.clone(),
                path,
            })
        })
        .collect();
    JsValue::Array(Rc::new(RefCell::new(children)))
}

fn text_content_raw(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        Node::Element(el) => el
            .children
            .iter()
            .map(text_content_raw)
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn inner_html(node: &Node) -> String {
    match node {
        Node::Text(text) => escape_html(text),
        Node::Element(el) => el
            .children
            .iter()
            .map(outer_html)
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn outer_html(node: &Node) -> String {
    match node {
        Node::Text(text) => escape_html(text),
        Node::Element(el) => {
            if el.tag == "#document" {
                return inner_html(node);
            }
            let mut out = String::new();
            out.push('<');
            out.push_str(&el.tag);
            for (k, v) in &el.attrs {
                out.push(' ');
                out.push_str(k);
                out.push_str("=\"");
                out.push_str(&escape_attr(v));
                out.push('"');
            }
            out.push('>');
            out.push_str(&inner_html(node));
            out.push_str("</");
            out.push_str(&el.tag);
            out.push('>');
            out
        }
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn escape_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}

pub fn browser_run_scripts_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!(
            "browser_run_scripts: expected 1 arg, got {}",
            args.len()
        ));
    }
    let Value::Str(html) = &args[0] else {
        return Err(format!(
            "browser_run_scripts: expected str, got {}",
            args[0].type_name()
        ));
    };
    result_to_value(run_html_scripts(html)?)
}

pub fn browser_eval_js_to_value(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(format!(
            "browser_eval_js: expected 2 args, got {}",
            args.len()
        ));
    }
    let Value::Str(html) = &args[0] else {
        return Err(format!(
            "browser_eval_js: html must be str, got {}",
            args[0].type_name()
        ));
    };
    let Value::Str(script) = &args[1] else {
        return Err(format!(
            "browser_eval_js: script must be str, got {}",
            args[1].type_name()
        ));
    };
    result_to_value(eval_with_dom(html, script)?)
}

fn result_to_value(result: BrowserJsResult) -> Result<Value, String> {
    let mut map = HashMap::new();
    map.insert("dom".into(), document_value(&result.document));
    map.insert("value".into(), js::js_to_tether(&result.value));
    map.insert(
        "console".into(),
        Value::List(Rc::new(RefCell::new(
            result
                .console
                .into_iter()
                .map(|s| Value::Str(Rc::new(s)))
                .collect(),
        ))),
    );
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

fn document_value(document: &browser::Document) -> Value {
    Value::List(Rc::new(RefCell::new(
        document.children.iter().map(node_value).collect(),
    )))
}

fn node_value(node: &Node) -> Value {
    let mut map = HashMap::new();
    match node {
        Node::Text(text) => {
            map.insert("type".into(), Value::Str(Rc::new("text".into())));
            map.insert("text".into(), Value::Str(Rc::new(text.clone())));
        }
        Node::Element(el) => {
            map.insert("type".into(), Value::Str(Rc::new("element".into())));
            map.insert("tag".into(), Value::Str(Rc::new(el.tag.clone())));
            map.insert(
                "attrs".into(),
                Value::Map(Rc::new(RefCell::new(
                    el.attrs
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::Str(Rc::new(v.clone()))))
                        .collect(),
                ))),
            );
            map.insert(
                "children".into(),
                Value::List(Rc::new(RefCell::new(
                    el.children.iter().map(node_value).collect(),
                ))),
            );
        }
    }
    Value::Map(Rc::new(RefCell::new(map)))
}

fn child_element_count(node: &Node) -> usize {
    match node {
        Node::Element(el) => el
            .children
            .iter()
            .filter(|child| matches!(child, Node::Element(_)))
            .count(),
        Node::Text(_) => 0,
    }
}

pub fn compatibility_report_to_value(_args: &[Value]) -> Result<Value, String> {
    let features = [
        "document",
        "window",
        "self",
        "selectors",
        "attributes",
        "textContent",
        "innerHTML",
        "createElement",
        "append",
        "remove",
        "events",
        "this",
        "typeof",
        "functionExpressions",
        "forLoops",
        "location",
        "URL",
        "URLSearchParams",
        "navigator",
        "setTimeout",
        "clearTimeout",
        "deterministicTimers",
        "localStorage",
        "sessionStorage",
        "Storage.getItem",
        "Storage.setItem",
        "Storage.removeItem",
        "Storage.clear",
        "Storage.key",
        "Storage.length",
        "fetch",
        "Headers",
        "Response",
        "Response.text",
        "Response.json",
    ];
    Ok(Value::List(Rc::new(RefCell::new(
        features
            .into_iter()
            .map(|s| Value::Str(Rc::new(s.into())))
            .collect(),
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scripts_can_read_document_and_console_log() {
        let html = "<main id='app'><h1>Hello</h1><script>console.log(document.getElementById('app').children.length); document.querySelector('h1').textContent;</script></main>";
        let result = run_html_scripts(html).unwrap();
        assert_eq!(result.value, JsValue::String("Hello".into()));
        assert_eq!(result.console, vec!["2".to_string()]);
    }

    #[test]
    fn eval_with_dom_exposes_selectors_and_attributes() {
        let result = eval_with_dom("<p class='note' id='x'>Hi</p>", "let p=document.querySelector('.note'); p.setAttribute('data-ok','yes'); p.getAttribute('id') + ':' + p.textContent;").unwrap();
        assert_eq!(result.value, JsValue::String("x:Hi".into()));
        match &result.document.children[0] {
            Node::Element(el) => assert_eq!(el.attrs.get("data-ok"), Some(&"yes".to_string())),
            Node::Text(_) => panic!("expected element"),
        }
    }

    #[test]
    fn dom_property_assignment_and_mutation_apis_update_document() {
        let result = eval_with_dom(
            "<main id='app'><p>old</p></main>",
            "let app=document.getElementById('app'); let p=document.querySelector('p'); p.textContent='new'; let span=document.createElement('span'); span.textContent='!'; app.appendChild(span); document.getElementById('app').children.length;",
        ).unwrap();
        assert_eq!(result.value, JsValue::Number(2.0));
        let text = result
            .document
            .children
            .iter()
            .map(browser::text_content)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("new"));
        assert!(text.contains("!"));
    }

    #[test]
    fn event_listeners_property_handlers_this_and_event_target_work() {
        let result = eval_with_dom(
            "<button id='go'>old</button>",
            "let btn=document.getElementById('go'); let seen=''; btn.addEventListener('click', function(e){ seen=e.type + ':' + e.target.id + ':' + this.id; this.textContent='clicked'; }); btn.click(); seen + ':' + document.getElementById('go').textContent;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("click:go:go:clicked".into()));

        let result = eval_with_dom(
            "<button id='go'>old</button>",
            "let btn=document.getElementById('go'); btn.onclick=function(e){ this.setAttribute('data-clicked', e.type); }; btn.dispatchEvent({type:'click'}); document.getElementById('go').getAttribute('data-clicked');",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("click".into()));
    }

    #[test]
    fn events_bubble_with_current_target_and_default_prevention() {
        let result = eval_with_dom(
            "<form id='form'><button id='go'>Send</button></form>",
            "let form=document.getElementById('form'); let btn=document.getElementById('go'); let seen=''; form.addEventListener('click', function(e){ seen=seen+'form:'+e.target.id+':'+e.currentTarget.id+':'+e.defaultPrevented+';'; }); btn.addEventListener('click', function(e){ seen=seen+'btn:'+e.target.id+':'+e.currentTarget.id+';'; e.preventDefault(); }); let ok=btn.dispatchEvent({type:'click'}); seen + ok;",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String("btn:go:go;form:go:form:true;false".into())
        );
    }

    #[test]
    fn inline_handler_false_prevents_default_and_parent_node_links_parent() {
        let result = eval_with_dom(
            "<div id='outer'><button id='go'>Send</button></div>",
            "let btn=document.getElementById('go'); btn.onclick=function(e){ return false; }; let ok=btn.dispatchEvent({type:'click'}); ok + ':' + btn.parentNode.id;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("false:outer".into()));
    }

    #[test]
    fn form_value_and_checked_properties_reflect_and_update_attributes() {
        let result = eval_with_dom(
            "<input id='name' value='Riley'><input id='agree' checked>",
            "let name=document.getElementById('name'); let agree=document.getElementById('agree'); let before=name.value+':'+agree.checked; name.value='Casey'; agree.checked=false; before + '|' + name.value + ':' + name.getAttribute('value') + ':' + agree.checked + ':' + agree.hasAttribute('checked');",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String("Riley:true|Casey:Casey:false:false".into())
        );
    }

    #[test]
    fn remove_event_listener_and_typeof_work() {
        let result = eval_with_dom(
            "<button>ok</button>",
            "let count=0; function inc(){ count=count+1; } let b=document.querySelector('button'); b.addEventListener('click', inc); b.removeEventListener('click', inc); b.click(); count + ':' + typeof missingName + ':' + typeof document;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("0:undefined:object".into()));
    }

    #[test]
    fn location_and_navigator_globals_are_available() {
        let result = eval_with_dom(
            "<main></main>",
            "navigator.userAgent.length > 0 && location.pathname == '/' && window.location.origin == 'http://localhost' && window.navigator.language == 'en-US';",
        ).unwrap();
        assert_eq!(result.value, JsValue::Bool(true));
    }

    #[test]
    fn url_and_url_search_params_are_available() {
        let result = eval_with_dom(
            "<main></main>",
            "let u=URL('/docs/page?x=1#top', 'https://example.com/root/index.html'); let p=URLSearchParams('a=1&b=two+words'); p.set('a','2'); p.append('c','3'); u.origin + '|' + u.pathname + '|' + u.searchParams.get('x') + '|' + p.get('b') + '|' + p.has('c') + '|' + p.toString();",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String(
                "https://example.com|/docs/page|1|two words|true|b=two+words&a=2&c=3".into()
            )
        );
    }

    #[test]
    fn location_assign_and_replace_update_location_fields() {
        let result = eval_with_dom(
            "<main></main>",
            "location.assign('/next?ok=1#frag'); let first=location.href + '|' + location.pathname + '|' + location.search + '|' + location.hash; location.replace('child'); first + '>' + location.href;",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String(
                "http://localhost/next?ok=1#frag|/next|?ok=1|#frag>http://localhost/child".into()
            )
        );
    }

    #[test]
    fn window_self_and_storage_globals_are_available() {
        let result = eval_with_dom(
            "<main></main>",
            "window.window === window && self === window && window.localStorage === localStorage && window.sessionStorage === sessionStorage;",
        ).unwrap();
        assert_eq!(result.value, JsValue::Bool(true));
    }

    #[test]
    fn classic_for_loop_supports_node_list_iteration() {
        let result = eval_with_dom(
            "<ul><li>A</li><li>B</li><li>C</li></ul>",
            "let items=document.querySelectorAll('li'); let text=''; for (let i=0; i<items.length; i=i+1) { text = text + items[i].textContent; } text;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("ABC".into()));
    }

    #[test]
    fn set_timeout_callbacks_drain_after_script_fifo() {
        let result = eval_with_dom(
            "<button id='go'>old</button>",
            "let order=''; setTimeout(function(){ order=order+'A'; document.getElementById('go').textContent='done'; }, 50); setTimeout(function(){ order=order+'B'; }, 0); order='sync'; order;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("sync".into()));
        assert_eq!(browser::text_content(&result.document.children[0]), "done");

        let result = eval_with_dom(
            "<main></main>",
            "let order=''; setTimeout(function(){ order=order+'A'; }, 10); setTimeout(function(){ order=order+'B'; console.log(order); }, 0); 'sync';",
        ).unwrap();
        assert_eq!(result.console, vec!["AB".to_string()]);
    }

    #[test]
    fn clear_timeout_cancels_pending_callback_and_timeout_args_work() {
        let result = eval_with_dom(
            "<main></main>",
            "let seen=''; let first=setTimeout(function(){ seen='bad'; }, 0); clearTimeout(first); window.setTimeout(function(a,b){ seen=a+b; console.log(this.navigator.language + ':' + seen); }, 0, 'O', 'K'); 'sync';",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("sync".into()));
        assert_eq!(result.console, vec!["en-US:OK".to_string()]);
    }

    #[test]
    fn local_storage_implements_minimal_storage_api() {
        let result = eval_with_dom(
            "<main></main>",
            "localStorage.setItem('a', 1); localStorage.setItem('b', 'two'); localStorage.setItem('a', 'one'); let before=localStorage.length + ':' + localStorage.key(0) + ':' + localStorage.getItem('a') + ':' + localStorage.getItem('missing'); localStorage.removeItem('b'); let after=localStorage.length + ':' + localStorage.key(1); localStorage.clear(); before + '|' + after + '|' + localStorage.length + ':' + localStorage.getItem('a');",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String("2:a:one:null|1:null|0:null".into())
        );
    }

    #[test]
    fn session_storage_is_separate_from_local_storage_and_per_eval() {
        let result = eval_with_dom(
            "<main></main>",
            "localStorage.setItem('shared', 'local'); sessionStorage.setItem('shared', 'session'); localStorage.getItem('shared') + ':' + sessionStorage.getItem('shared') + ':' + localStorage.length + ':' + sessionStorage.length;",
        ).unwrap();
        assert_eq!(result.value, JsValue::String("local:session:1:1".into()));

        let result = eval_with_dom(
            "<main></main>",
            "localStorage.getItem('shared') === null && sessionStorage.length === 0;",
        )
        .unwrap();
        assert_eq!(result.value, JsValue::Bool(true));
    }

    #[test]
    fn compatibility_report_lists_storage_apis() {
        let report = compatibility_report_to_value(&[]).unwrap();
        let Value::List(items) = report else {
            panic!("expected list");
        };
        let features = items
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Str(s) => s.to_string(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>();
        assert!(features.contains(&"localStorage".to_string()));
        assert!(features.contains(&"sessionStorage".to_string()));
        assert!(features.contains(&"Storage.length".to_string()));
    }

    #[test]
    fn fetch_returns_response_like_object_with_helpers() {
        let result = eval_with_dom(
            "<main></main>",
            "let r=fetch('/api', {method:'post', body:'123', headers:{'X-Test':'yes'}}); r.ok + ':' + r.status + ':' + r.statusText + ':' + r.url + ':' + r.text() + ':' + r.json() + ':' + r.headers.get('x-test') + ':' + r.headers.get('CONTENT-TYPE');",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String("true:200:OK:/api:123:123:yes:text/plain;charset=utf-8".into())
        );
    }

    #[test]
    fn response_constructor_and_headers_normalize_names() {
        let result = eval_with_dom(
            "<main></main>",
            "let h=Headers({'Content-Type':'application/json','x-id':'a'}); h.append('X-ID','b'); let r=Response('{\\\"ok\\\":true}', {status:404, headers:h}); r.ok + ':' + r.status + ':' + r.statusText + ':' + r.headers.has('content-type') + ':' + r.headers.get('CONTENT-TYPE') + ':' + r.headers.get('x-id') + ':' + r.text();",
        ).unwrap();
        assert_eq!(
            result.value,
            JsValue::String("false:404:Not Found:true:application/json:a, b:{\"ok\":true}".into())
        );
    }

    #[test]
    fn compatibility_report_lists_fetch_response_apis() {
        let report = compatibility_report_to_value(&[]).unwrap();
        let Value::List(items) = report else {
            panic!("expected list");
        };
        let features = items
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Str(s) => s.to_string(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>();
        assert!(features.contains(&"fetch".to_string()));
        assert!(features.contains(&"Headers".to_string()));
        assert!(features.contains(&"Response".to_string()));
        assert!(features.contains(&"Response.text".to_string()));
        assert!(features.contains(&"Response.json".to_string()));
    }
}
