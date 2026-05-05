//! Live browser capability for agentic UI debugging.
//!
//! This module is the production-oriented seam between TetherScript and a real
//! browser engine. It deliberately does **not** try to implement Chromium in
//! tree. Instead, it exposes a capability-gated API that can delegate to a
//! CodeTether/browserctl-compatible HTTP service (which in turn drives Chrome
//! DevTools Protocol), while preserving deterministic in-tree browser APIs for
//! offline validation elsewhere in the project.
//!
//! Transport contract (MVP): POST JSON to `{endpoint}/<method>` with bodies such
//! as `{ "selector": "#app" }`. Responses may be either raw JSON values or
//! `{ "ok": true, "value": ... }` / `{ "ok": false, "error": "..." }`.
//! This keeps the authority independent of a particular CDP client crate and
//! lets CodeTether inject its existing browser infrastructure.

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::capability::Authority;
use crate::json;
use crate::value::{Runtime, Value};

const USER_AGENT: &str = "tetherscript-browser-cap/0.1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct BrowserAuthority {
    endpoint: String,
    allowed_origins: Vec<String>,
    allowed_scopes: HashSet<String>,
    path_prefix: Option<String>,
    storage_scope: Option<String>,
    human_approval: bool,
    timeout: Duration,
    trace: Rc<RefCell<BrowserTrace>>,
}

#[derive(Clone, Debug, Default)]
struct BrowserTrace {
    actions: Vec<TraceEvent>,
    observations: Vec<TraceEvent>,
    console_events: Vec<Value>,
    network_events: Vec<Value>,
    screenshots: Vec<TraceEvent>,
    dom_snapshots: Vec<Value>,
    storage_changes: Vec<TraceEvent>,
    errors: Vec<TraceEvent>,
}

#[derive(Clone, Debug)]
struct TraceEvent {
    timestamp_ms: i64,
    kind: String,
    method: String,
    data: Value,
}

impl BrowserAuthority {
    /// Create a browser authority scoped to a CodeTether/browserctl HTTP bridge.
    /// `endpoint` is typically something like `http://127.0.0.1:41707/browser`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(endpoint: &str, origins: Vec<String>, scopes: Vec<String>) -> Rc<dyn Authority> {
        Rc::new(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            allowed_origins: origins.into_iter().map(normalize_origin).collect(),
            allowed_scopes: scopes.into_iter().collect(),
            path_prefix: None,
            storage_scope: None,
            human_approval: false,
            timeout: DEFAULT_TIMEOUT,
            trace: Rc::new(RefCell::new(BrowserTrace::default())),
        })
    }

    pub fn all_scopes() -> Vec<String> {
        [
            "browser.navigate",
            "browser.interact",
            "browser.inspect.dom",
            "browser.inspect.network",
            "browser.inspect.console",
            "browser.inspect.storage",
            "browser.inspect.react",
            "browser.mutate.storage",
            "browser.replay.network",
            "browser.screenshot",
            "browser.visual",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn require_scope(&self, scope: &str) -> Result<(), String> {
        if self.allowed_scopes.contains(scope) {
            Ok(())
        } else {
            Err(format!("browser: scope `{}` not granted", scope))
        }
    }

    fn require_origin_url(&self, url: &str) -> Result<(), String> {
        if self.allowed_origins.is_empty() {
            return Ok(());
        }
        let parsed = ParsedUrl::parse(url)?;
        let origin = parsed.origin();
        if !self.allowed_origins.iter().any(|o| o == &origin) {
            return Err(format!(
                "browser: origin {} is not in allowed set {:?}",
                origin, self.allowed_origins
            ));
        }
        if let Some(prefix) = &self.path_prefix {
            if !parsed.path.starts_with(prefix) {
                return Err(format!(
                    "browser: path {} does not match required prefix {}",
                    parsed.path, prefix
                ));
            }
        }
        Ok(())
    }

    fn approval_check(&self, method: &str) -> Result<(), String> {
        if self.human_approval {
            Err(format!(
                "browser.{}: human approval gate is enabled; host approval integration required",
                method
            ))
        } else {
            Ok(())
        }
    }

    fn invoke_remote(&self, method: &str, payload: Value) -> Result<Value, String> {
        let body = json::encode_to_string(&payload)?;
        let url = format!("{}/{}", self.endpoint, method);
        let response = http_post_json(&url, &body, self.timeout)?;
        if response.trim().is_empty() {
            return Ok(Value::Nil);
        }
        let parsed = json::parse_str(&response)
            .map_err(|e| format!("browser.{}: invalid bridge JSON: {}", method, e))?;
        unwrap_bridge_response(method, parsed)
    }

    fn action(&self, method: &str, data: Value) {
        self.trace
            .borrow_mut()
            .actions
            .push(TraceEvent::new("action", method, data));
    }

    fn observe(&self, method: &str, value: &Value) {
        let mut trace = self.trace.borrow_mut();
        trace
            .observations
            .push(TraceEvent::new("observation", method, value.clone()));
        match method {
            "console_logs" | "console_errors" | "unhandled_rejections" | "runtime_exceptions" => {
                trace.console_events.push(value.clone())
            }
            "network_log" | "failed_requests" | "request" | "response" => {
                trace.network_events.push(value.clone())
            }
            "screenshot" | "screenshot_element" => {
                trace
                    .screenshots
                    .push(TraceEvent::new("screenshot", method, value.clone()))
            }
            "dom_snapshot" | "page_snapshot" => trace.dom_snapshots.push(value.clone()),
            _ => {}
        }
    }

    fn error(&self, method: &str, message: String) {
        self.trace.borrow_mut().errors.push(TraceEvent::new(
            "error",
            method,
            Value::Str(Rc::new(message)),
        ));
    }

    fn call_checked(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        let (scope, payload) = self.prepare_call(method, args)?;
        self.require_scope(scope)?;
        if is_mutating_or_external(method) {
            self.approval_check(method)?;
        }
        self.action(method, payload.clone());
        let result = self.invoke_remote(method, payload);
        match result {
            Ok(value) => {
                self.observe(method, &value);
                Ok(value)
            }
            Err(e) => {
                self.error(method, e.clone());
                Err(e)
            }
        }
    }

    fn prepare_call(&self, method: &str, args: &[Value]) -> Result<(&'static str, Value), String> {
        match method {
            "goto" => {
                let url = expect_str(method, args, 0)?;
                self.require_origin_url(&url)?;
                Ok(("browser.navigate", map_value(vec![("url", str_value(url))])))
            }
            "reload" | "back" | "forward" => {
                no_args(method, args).map(|_| ("browser.navigate", empty_map()))
            }
            "click" | "hover" | "focus" | "blur" | "screenshot_element" | "bounding_box"
            | "is_visible" => {
                let selector = expect_str(method, args, 0)?;
                let scope = if method == "screenshot_element" {
                    "browser.screenshot"
                } else if method == "bounding_box" || method == "is_visible" {
                    "browser.visual"
                } else {
                    "browser.interact"
                };
                Ok((scope, map_value(vec![("selector", str_value(selector))])))
            }
            "click_text" | "wait_for_text" | "find_visual_text" => {
                let text = expect_str(method, args, 0)?;
                let timeout = optional_int(args, 1, 30_000)?;
                let scope = if method == "find_visual_text" {
                    "browser.visual"
                } else if method == "click_text" {
                    "browser.interact"
                } else {
                    "browser.inspect.dom"
                };
                Ok((
                    scope,
                    map_value(vec![
                        ("text", str_value(text)),
                        ("timeout_ms", Value::Int(timeout)),
                    ]),
                ))
            }
            "type" => Ok((
                "browser.interact",
                map_value(vec![
                    ("selector", str_value(expect_str(method, args, 0)?)),
                    ("text", str_value(expect_str(method, args, 1)?)),
                ]),
            )),
            "press" => Ok((
                "browser.interact",
                map_value(vec![("key", str_value(expect_str(method, args, 0)?))]),
            )),
            "scroll" => Ok((
                "browser.interact",
                map_value(vec![
                    (
                        "target",
                        args.get(0).cloned().unwrap_or_else(|| str_value("window")),
                    ),
                    ("amount", Value::Int(expect_int(method, args, 1)?)),
                ]),
            )),
            "wait_for_selector" => Ok((
                "browser.inspect.dom",
                map_value(vec![
                    ("selector", str_value(expect_str(method, args, 0)?)),
                    ("timeout_ms", Value::Int(optional_int(args, 1, 30_000)?)),
                ]),
            )),
            "wait_for_url" => Ok((
                "browser.navigate",
                map_value(vec![
                    ("substring", str_value(expect_str(method, args, 0)?)),
                    ("timeout_ms", Value::Int(optional_int(args, 1, 30_000)?)),
                ]),
            )),
            "wait_for_network_idle" => Ok((
                "browser.inspect.network",
                map_value(vec![(
                    "timeout_ms",
                    Value::Int(optional_int(args, 0, 5_000)?),
                )]),
            )),
            "screenshot" => no_args(method, args).map(|_| ("browser.screenshot", empty_map())),
            "dom_snapshot" | "page_snapshot" => {
                no_args(method, args).map(|_| ("browser.inspect.dom", empty_map()))
            }
            "console_logs"
            | "console_errors"
            | "unhandled_rejections"
            | "runtime_exceptions"
            | "source_mapped_stack_traces" => {
                no_args(method, args).map(|_| ("browser.inspect.console", empty_map()))
            }
            "network_log" | "failed_requests" => {
                no_args(method, args).map(|_| ("browser.inspect.network", empty_map()))
            }
            "request" | "response" => Ok((
                "browser.inspect.network",
                map_value(vec![("id", expect_id_value(method, args, 0)?)]),
            )),
            "replay_request" => {
                let mut entries = vec![("id", expect_id_value(method, args, 0)?)];
                if let Some(patch) = args.get(1) {
                    entries.push(("body_patch", patch.clone()));
                }
                Ok(("browser.replay.network", map_value(entries)))
            }
            "wait_for_request" | "wait_for_response" => Ok((
                "browser.inspect.network",
                map_value(vec![
                    ("url_pattern", str_value(expect_str(method, args, 0)?)),
                    ("timeout_ms", Value::Int(optional_int(args, 1, 30_000)?)),
                ]),
            )),
            "cookies" | "local_storage" | "session_storage" | "indexed_db_summary" => {
                no_args(method, args).map(|_| {
                    (
                        "browser.inspect.storage",
                        storage_payload(&self.storage_scope),
                    )
                })
            }
            "set_cookie" => Ok((
                "browser.mutate.storage",
                map_value(vec![(
                    "cookie",
                    args.get(0)
                        .cloned()
                        .ok_or_else(|| "set_cookie expects cookie map".to_string())?,
                )]),
            )),
            "set_local_storage" => Ok((
                "browser.mutate.storage",
                map_value(vec![
                    ("key", str_value(expect_str(method, args, 0)?)),
                    ("value", str_value(expect_str(method, args, 1)?)),
                ]),
            )),
            "clear_storage" => no_args(method, args).map(|_| {
                (
                    "browser.mutate.storage",
                    storage_payload(&self.storage_scope),
                )
            }),
            "react.detect"
            | "react.version"
            | "react.component_tree"
            | "react.errors"
            | "react.hydration_warnings"
            | "react.suspense_boundaries"
            | "frameworks" => no_args(method, args).map(|_| ("browser.inspect.react", empty_map())),
            "react.component_for_selector" => Ok((
                "browser.inspect.react",
                map_value(vec![("selector", str_value(expect_str(method, args, 0)?))]),
            )),
            "react.props" | "react.state" | "react.hooks" | "react.owner_stack" => Ok((
                "browser.inspect.react",
                map_value(vec![("component_id", expect_id_value(method, args, 0)?)]),
            )),
            "compare_screenshots" | "visual_diff" => Ok((
                "browser.visual",
                map_value(vec![
                    (
                        "before",
                        args.get(0)
                            .cloned()
                            .ok_or_else(|| format!("{} expects before", method))?,
                    ),
                    (
                        "after",
                        args.get(1)
                            .cloned()
                            .ok_or_else(|| format!("{} expects after", method))?,
                    ),
                ]),
            )),
            "find_element_at" => Ok((
                "browser.visual",
                map_value(vec![
                    ("x", Value::Int(expect_int(method, args, 0)?)),
                    ("y", Value::Int(expect_int(method, args, 1)?)),
                ]),
            )),
            "trace"
            | "export_trace_json"
            | "export_har"
            | "agent_summary"
            | "minimal_reproduction_script" => {
                no_args(method, args).map(|_| ("browser.inspect.dom", empty_map()))
            }
            _ => Err(format!("browser: no method `{}`", method)),
        }
    }
}

impl Authority for BrowserAuthority {
    fn narrow(&self, params: &Value) -> Result<Rc<dyn Authority>, String> {
        let map = match params {
            Value::Map(m) => m.borrow(),
            _ => return Err("browser.narrow: expected params map".into()),
        };
        let mut next = self.clone();
        if let Some(v) = map.get("origins") {
            let requested = string_list(v, "browser.narrow origins")?
                .into_iter()
                .map(normalize_origin)
                .collect::<Vec<_>>();
            if !self.allowed_origins.is_empty()
                && !requested
                    .iter()
                    .all(|r| self.allowed_origins.iter().any(|o| o == r))
            {
                return Err("browser.narrow: requested origins are not a subset".into());
            }
            next.allowed_origins = requested;
        }
        if let Some(v) = map.get("scopes") {
            let requested: HashSet<String> = string_list(v, "browser.narrow scopes")?
                .into_iter()
                .collect();
            if !requested.iter().all(|s| self.allowed_scopes.contains(s)) {
                return Err("browser.narrow: requested scopes are not a subset".into());
            }
            next.allowed_scopes = requested;
        }
        if let Some(Value::Str(p)) = map.get("path_prefix") {
            let p = (**p).clone();
            if let Some(current) = &self.path_prefix {
                if !p.starts_with(current) {
                    return Err("browser.narrow: path_prefix must extend current prefix".into());
                }
            }
            next.path_prefix = Some(p);
        }
        if let Some(Value::Str(s)) = map.get("storage_scope") {
            next.storage_scope = Some((**s).clone());
        }
        if let Some(Value::Bool(b)) = map.get("human_approval") {
            next.human_approval = *b || self.human_approval;
        }
        Ok(Rc::new(next))
    }

    fn invoke(&self, _rt: &mut dyn Runtime, method: &str, args: &[Value]) -> Result<Value, String> {
        match method {
            "describe" => Ok(self.describe()),
            "trace" | "export_trace_json" => Ok(self.trace_value()),
            "export_har" => self
                .call_checked("export_har", args)
                .or_else(|_| Ok(self.har_value())),
            "agent_summary" => self
                .call_checked("agent_summary", args)
                .or_else(|_| Ok(self.agent_summary())),
            "minimal_reproduction_script" => self
                .call_checked("minimal_reproduction_script", args)
                .or_else(|_| Ok(self.minimal_repro())),
            _ => self.call_checked(method, args),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BrowserAuthority {
    fn describe(&self) -> Value {
        map_value(vec![
            ("endpoint", str_value(self.endpoint.clone())),
            ("origins", list_str(self.allowed_origins.clone())),
            ("scopes", list_str(sorted_set(&self.allowed_scopes))),
            ("path_prefix", opt_str(&self.path_prefix)),
            ("storage_scope", opt_str(&self.storage_scope)),
            ("human_approval", Value::Bool(self.human_approval)),
        ])
    }

    fn trace_value(&self) -> Value {
        let t = self.trace.borrow();
        map_value(vec![
            ("actions", trace_events(&t.actions)),
            ("observations", trace_events(&t.observations)),
            (
                "console_events",
                Value::List(Rc::new(RefCell::new(t.console_events.clone()))),
            ),
            (
                "network_events",
                Value::List(Rc::new(RefCell::new(t.network_events.clone()))),
            ),
            ("screenshots", trace_events(&t.screenshots)),
            (
                "dom_snapshots",
                Value::List(Rc::new(RefCell::new(t.dom_snapshots.clone()))),
            ),
            ("storage_changes", trace_events(&t.storage_changes)),
            ("errors", trace_events(&t.errors)),
            (
                "timestamps",
                map_value(vec![("exported_at_ms", Value::Int(now_ms()))]),
            ),
        ])
    }

    fn har_value(&self) -> Value {
        let t = self.trace.borrow();
        map_value(vec![(
            "log",
            map_value(vec![
                ("version", str_value("1.2")),
                (
                    "creator",
                    map_value(vec![
                        ("name", str_value("TetherScript BrowserAuthority")),
                        ("version", str_value("0.1")),
                    ]),
                ),
                (
                    "entries",
                    Value::List(Rc::new(RefCell::new(t.network_events.clone()))),
                ),
            ]),
        )])
    }

    fn agent_summary(&self) -> Value {
        let t = self.trace.borrow();
        map_value(vec![
            ("action_count", Value::Int(t.actions.len() as i64)),
            ("observation_count", Value::Int(t.observations.len() as i64)),
            (
                "console_event_count",
                Value::Int(t.console_events.len() as i64),
            ),
            (
                "network_event_count",
                Value::Int(t.network_events.len() as i64),
            ),
            ("error_count", Value::Int(t.errors.len() as i64)),
        ])
    }

    fn minimal_repro(&self) -> Value {
        let t = self.trace.borrow();
        let mut lines = vec!["fn repro(browser) {".to_string()];
        for event in &t.actions {
            lines.push(format!("    // {}", event.method));
        }
        lines.push("    return browser.trace()?".to_string());
        lines.push("}".to_string());
        str_value(lines.join("\n"))
    }
}

impl TraceEvent {
    fn new(kind: &str, method: &str, data: Value) -> Self {
        Self {
            timestamp_ms: now_ms(),
            kind: kind.into(),
            method: method.into(),
            data,
        }
    }
    fn value(&self) -> Value {
        map_value(vec![
            ("timestamp_ms", Value::Int(self.timestamp_ms)),
            ("kind", str_value(self.kind.clone())),
            ("method", str_value(self.method.clone())),
            ("data", self.data.clone()),
        ])
    }
}

fn unwrap_bridge_response(method: &str, parsed: Value) -> Result<Value, String> {
    if let Value::Map(m) = &parsed {
        let b = m.borrow();
        if let Some(Value::Bool(ok)) = b.get("ok") {
            if *ok {
                return Ok(b
                    .get("value")
                    .cloned()
                    .or_else(|| b.get("result").cloned())
                    .unwrap_or(Value::Nil));
            }
            let msg = b
                .get("error")
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown bridge error".into());
            return Err(format!("browser.{}: {}", method, msg));
        }
    }
    Ok(parsed)
}

fn is_mutating_or_external(method: &str) -> bool {
    matches!(
        method,
        "goto"
            | "click"
            | "click_text"
            | "type"
            | "press"
            | "hover"
            | "focus"
            | "blur"
            | "scroll"
            | "reload"
            | "back"
            | "forward"
            | "set_cookie"
            | "set_local_storage"
            | "clear_storage"
            | "replay_request"
    )
}
fn no_args(method: &str, args: &[Value]) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("browser.{} expects no arguments", method))
    }
}
fn expect_str(method: &str, args: &[Value], index: usize) -> Result<String, String> {
    match args.get(index) {
        Some(Value::Str(s)) => Ok((**s).clone()),
        Some(v) => Err(format!(
            "browser.{} arg {} must be str, got {}",
            method,
            index + 1,
            v.type_name()
        )),
        None => Err(format!("browser.{} missing arg {}", method, index + 1)),
    }
}
fn expect_int(method: &str, args: &[Value], index: usize) -> Result<i64, String> {
    match args.get(index) {
        Some(Value::Int(n)) => Ok(*n),
        Some(Value::Float(f)) => Ok(*f as i64),
        Some(v) => Err(format!(
            "browser.{} arg {} must be int, got {}",
            method,
            index + 1,
            v.type_name()
        )),
        None => Err(format!("browser.{} missing arg {}", method, index + 1)),
    }
}
fn optional_int(args: &[Value], index: usize, default: i64) -> Result<i64, String> {
    match args.get(index) {
        None => Ok(default),
        Some(Value::Int(n)) => Ok(*n),
        Some(Value::Float(f)) => Ok(*f as i64),
        Some(v) => Err(format!("timeout/amount must be int, got {}", v.type_name())),
    }
}
fn expect_id_value(method: &str, args: &[Value], index: usize) -> Result<Value, String> {
    args.get(index)
        .cloned()
        .ok_or_else(|| format!("browser.{} missing id", method))
}
fn str_value(s: impl Into<String>) -> Value {
    Value::Str(Rc::new(s.into()))
}
fn empty_map() -> Value {
    map_value(Vec::new())
}
fn map_value(entries: Vec<(&str, Value)>) -> Value {
    Value::Map(Rc::new(RefCell::new(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )))
}
fn list_str(values: Vec<String>) -> Value {
    Value::List(Rc::new(RefCell::new(
        values.into_iter().map(str_value).collect(),
    )))
}
fn opt_str(value: &Option<String>) -> Value {
    value.clone().map(str_value).unwrap_or(Value::Nil)
}
fn storage_payload(scope: &Option<String>) -> Value {
    map_value(vec![("storage_scope", opt_str(scope))])
}
fn trace_events(events: &[TraceEvent]) -> Value {
    Value::List(Rc::new(RefCell::new(
        events.iter().map(|e| e.value()).collect(),
    )))
}
fn sorted_set(set: &HashSet<String>) -> Vec<String> {
    let mut v: Vec<_> = set.iter().cloned().collect();
    v.sort();
    v
}
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
fn string_list(v: &Value, label: &str) -> Result<Vec<String>, String> {
    match v {
        Value::List(xs) => xs
            .borrow()
            .iter()
            .map(|x| match x {
                Value::Str(s) => Ok((**s).clone()),
                other => Err(format!(
                    "{} entries must be strings, got {}",
                    label,
                    other.type_name()
                )),
            })
            .collect(),
        other => Err(format!("{} must be list, got {}", label, other.type_name())),
    }
}

fn http_post_json(url: &str, body: &str, timeout: Duration) -> Result<String, String> {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;

    let parsed = BridgeUrl::parse(url)?;
    let mut stream = TcpStream::connect((parsed.host.as_str(), parsed.port)).map_err(|e| {
        format!(
            "browser bridge: connect to {}:{} failed: {}",
            parsed.host, parsed.port, e
        )
    })?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("browser bridge: set read timeout failed: {}", e))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("browser bridge: set write timeout failed: {}", e))?;
    write!(stream, "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: {}\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}", parsed.target, parsed.host_header, USER_AGENT, body.as_bytes().len(), body)
        .and_then(|_| stream.flush())
        .map_err(|e| format!("browser bridge: write request failed: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .map_err(|e| format!("browser bridge: read status failed: {}", e))?;
    let mut parts = status_line.split_whitespace();
    let _http = parts.next();
    let status: u16 = parts
        .next()
        .ok_or_else(|| "browser bridge: missing HTTP status".to_string())?
        .parse()
        .map_err(|_| {
            format!(
                "browser bridge: bad HTTP status line {}",
                status_line.trim()
            )
        })?;
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("browser bridge: read header failed: {}", e))?;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().ok();
            }
        }
    }
    let mut bytes = Vec::new();
    match content_length {
        Some(n) => {
            let mut take = reader.take(n as u64);
            take.read_to_end(&mut bytes)
                .map_err(|e| format!("browser bridge: read body failed: {}", e))?;
        }
        None => {
            reader
                .read_to_end(&mut bytes)
                .map_err(|e| format!("browser bridge: read body failed: {}", e))?;
        }
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if !(200..300).contains(&status) {
        return Err(format!("browser bridge returned HTTP {}: {}", status, text));
    }
    Ok(text)
}

struct BridgeUrl {
    host: String,
    port: u16,
    host_header: String,
    target: String,
}
impl BridgeUrl {
    fn parse(input: &str) -> Result<Self, String> {
        let rest = input
            .strip_prefix("http://")
            .ok_or_else(|| "browser bridge endpoint must be http:// in std-only MVP".to_string())?;
        let split_at = rest
            .char_indices()
            .find_map(|(idx, ch)| matches!(ch, '/' | '?').then_some(idx))
            .unwrap_or(rest.len());
        let authority = &rest[..split_at];
        let suffix = &rest[split_at..];
        if authority.is_empty() {
            return Err("browser bridge URL missing host".into());
        }
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (
                h.to_string(),
                p.parse()
                    .map_err(|_| "browser bridge bad port".to_string())?,
            ),
            _ => (authority.to_string(), 80),
        };
        let host_header = if port == 80 {
            host.clone()
        } else {
            format!("{}:{}", host, port)
        };
        let target = if suffix.is_empty() {
            "/".to_string()
        } else if suffix.starts_with('?') {
            format!("/{}", suffix)
        } else {
            suffix.to_string()
        };
        Ok(Self {
            host,
            port,
            host_header,
            target,
        })
    }
}

struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
}
impl ParsedUrl {
    fn parse(url: &str) -> Result<Self, String> {
        let (scheme, rest) = url
            .split_once("://")
            .ok_or_else(|| format!("browser: url `{}` has no scheme", url))?;
        let scheme = scheme.to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(format!("browser: unsupported url scheme `{}`", scheme));
        }
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (
                h.to_ascii_lowercase(),
                Some(p.parse().map_err(|_| "browser: bad port".to_string())?),
            ),
            _ => (authority.to_ascii_lowercase(), None),
        };
        Ok(Self {
            scheme,
            host,
            port,
            path: path.to_string(),
        })
    }
    fn origin(&self) -> String {
        match self.port {
            Some(p) => format!("{}://{}:{}", self.scheme, self.host, p),
            None => format!("{}://{}", self.scheme, self.host),
        }
    }
}
fn normalize_origin(s: String) -> String {
    ParsedUrl::parse(s.trim_end_matches('/'))
        .map(|p| p.origin())
        .unwrap_or(s)
}
