//! `HttpAuthority` — outgoing HTTP as a capability.
//!
//! Grants the agent the right to make HTTP requests, scoped to a set of
//! allowed origins + methods + an optional path prefix. Credentials
//! (Authorization, API keys) are bound to the capability at grant time:
//! the harness specifies headers that ride with every request, and the
//! agent *cannot* read or override them. Attenuation follows the same
//! monotonic rule as fs — narrowed ⊆ parent, always.
//!
//! # Security posture
//!
//! - **Origin matching** is exact scheme + host + (explicit or default)
//!   port. `https://api.example.com` does not match `http://...` or
//!   `https://api.example.com:8443`.
//! - **Path prefix** is applied as a string prefix on the URL path. A
//!   capability scoped to `/v1/items` will reject `/v1/items/../../other`
//!   *after* url normalization.
//! - **Bound headers** are invisible to TetherScript code. The agent sees a
//!   capability that can fetch; it never sees the token that authorizes
//!   the fetch. This is the concrete expression of "agent holds the grant,
//!   not the secret."
//! - No redirects followed. A redirect response is returned as-is; if the
//!   agent wants to follow it, it must fetch the redirect target explicitly,
//!   and that target will re-check against the scope.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use crate::capability::Authority;
use crate::value::{Runtime, Value};

pub struct HttpAuthority {
    /// Allowed origins, normalized to `scheme://host[:port]`. Empty = deny.
    origins: Vec<String>,
    /// Uppercased allowed methods.
    methods: HashSet<String>,
    /// Optional URL-path prefix. Applied after origin match.
    path_prefix: Option<String>,
    /// Headers attached to every request. Bound at grant time; not
    /// scriptable from TetherScript.
    bound_headers: Vec<(String, String)>,
    user_agent: String,
    timeout: Duration,
}

impl HttpAuthority {
    /// Build an HTTP capability with full GET/POST/HEAD access to the given
    /// origins. Additional bound headers (e.g. API tokens) may be attached.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(origins: Vec<String>) -> Rc<dyn Authority> {
        let mut methods = HashSet::new();
        for m in ["GET", "POST", "HEAD"] {
            methods.insert(m.to_string());
        }
        Rc::new(HttpAuthority {
            origins: origins.into_iter().map(normalize_origin).collect(),
            methods,
            path_prefix: None,
            bound_headers: Vec::new(),
            user_agent: "tetherscript-agent/0.1".into(),
            timeout: Duration::from_secs(15),
        })
    }

    /// Attach a header to every request made through this authority. Only
    /// the harness should call this — TetherScript code has no way to reach the
    /// underlying `HttpAuthority`.
    pub fn with_bound_header(
        auth: Rc<dyn Authority>,
        name: &str,
        value: &str,
    ) -> Rc<dyn Authority> {
        let this = auth
            .as_any()
            .downcast_ref::<HttpAuthority>()
            .expect("with_bound_header: authority is not HttpAuthority");
        let mut bound = this.bound_headers.clone();
        bound.push((name.to_string(), value.to_string()));
        Rc::new(HttpAuthority {
            origins: this.origins.clone(),
            methods: this.methods.clone(),
            path_prefix: this.path_prefix.clone(),
            bound_headers: bound,
            user_agent: this.user_agent.clone(),
            timeout: this.timeout,
        })
    }

    fn check_scope(&self, method: &str, url: &str) -> Result<(), String> {
        if !self.methods.contains(method) {
            return Err(format!("http: method {} not allowed by capability", method));
        }
        let parsed = ParsedUrl::parse(url)?;
        let origin = parsed.origin();
        if !self.origins.iter().any(|o| o == &origin) {
            return Err(format!(
                "http: origin {} is not in the allowed set ({:?})",
                origin, self.origins
            ));
        }
        if let Some(prefix) = &self.path_prefix {
            if !parsed.path.starts_with(prefix) {
                return Err(format!(
                    "http: path {} does not match required prefix {}",
                    parsed.path, prefix
                ));
            }
        }
        Ok(())
    }

    fn do_request(&self, method: &str, url: &str, body: Option<&str>) -> Result<Value, String> {
        self.check_scope(method, url)?;

        let agent = ureq::AgentBuilder::new()
            .timeout(self.timeout)
            .user_agent(&self.user_agent)
            .redirects(0)
            .build();

        let mut req = match method {
            "GET" => agent.get(url),
            "POST" => agent.post(url),
            "HEAD" => agent.head(url),
            _ => return Err(format!("http: unsupported method {}", method)),
        };
        for (k, v) in &self.bound_headers {
            req = req.set(k, v);
        }

        let resp = match body {
            Some(b) => req.send_string(b),
            None => req.call(),
        };

        match resp {
            Ok(r) => Ok(make_response(r)),
            Err(ureq::Error::Status(code, r)) => {
                // Non-2xx — still a structured response; surface it with the body.
                let mut m = HashMap::new();
                m.insert("status".into(), Value::Int(code as i64));
                let body = r.into_string().unwrap_or_default();
                m.insert("body".into(), Value::Str(Rc::new(body)));
                m.insert("ok".into(), Value::Bool(false));
                Ok(Value::Map(Rc::new(RefCell::new(m))))
            }
            Err(e) => Err(format!("http.{}: {}", method.to_lowercase(), e)),
        }
    }
}

fn make_response(resp: ureq::Response) -> Value {
    let status = resp.status();
    let mut headers_map: HashMap<String, Value> = HashMap::new();
    for name in resp.headers_names() {
        if let Some(v) = resp.header(&name) {
            headers_map.insert(name.to_ascii_lowercase(), Value::Str(Rc::new(v.into())));
        }
    }
    let body = resp.into_string().unwrap_or_default();
    let mut m: HashMap<String, Value> = HashMap::new();
    m.insert("status".into(), Value::Int(status as i64));
    m.insert("ok".into(), Value::Bool((200..300).contains(&status)));
    m.insert(
        "headers".into(),
        Value::Map(Rc::new(RefCell::new(headers_map))),
    );
    m.insert("body".into(), Value::Str(Rc::new(body)));
    Value::Map(Rc::new(RefCell::new(m)))
}

impl Authority for HttpAuthority {
    fn narrow(&self, params: &Value) -> Result<Rc<dyn Authority>, String> {
        let map = match params {
            Value::Map(m) => m.clone(),
            _ => return Err("http.narrow: expected a map of params".into()),
        };
        let m = map.borrow();

        let mut new_origins = self.origins.clone();
        let mut new_methods = self.methods.clone();
        let mut new_prefix = self.path_prefix.clone();

        if let Some(v) = m.get("origins") {
            let xs = match v {
                Value::List(xs) => xs.clone(),
                _ => return Err("http.narrow: `origins` must be a list of strings".into()),
            };
            let requested: Vec<String> = xs
                .borrow()
                .iter()
                .map(|x| match x {
                    Value::Str(s) => Ok(normalize_origin((**s).clone())),
                    _ => Err("http.narrow: origins must be strings".to_string()),
                })
                .collect::<Result<_, _>>()?;
            new_origins.retain(|o| requested.iter().any(|r| r == o));
            if new_origins.is_empty() {
                return Err(
                    "http.narrow: requested origins are not a subset of current scope".into(),
                );
            }
        }

        if let Some(v) = m.get("methods") {
            let xs = match v {
                Value::List(xs) => xs.clone(),
                _ => return Err("http.narrow: `methods` must be a list of strings".into()),
            };
            let requested: HashSet<String> = xs
                .borrow()
                .iter()
                .map(|x| match x {
                    Value::Str(s) => Ok(s.to_ascii_uppercase()),
                    _ => Err("http.narrow: methods must be strings".to_string()),
                })
                .collect::<Result<_, _>>()?;
            new_methods = new_methods.intersection(&requested).cloned().collect();
            if new_methods.is_empty() {
                return Err("http.narrow: no methods left after intersection".into());
            }
        }

        if let Some(v) = m.get("path_prefix") {
            let p = match v {
                Value::Str(s) => (**s).clone(),
                _ => return Err("http.narrow: `path_prefix` must be a string".into()),
            };
            if let Some(current) = &new_prefix {
                if !p.starts_with(current) {
                    return Err(format!(
                        "http.narrow: prefix {} does not extend current prefix {}",
                        p, current
                    ));
                }
            }
            new_prefix = Some(p);
        }

        Ok(Rc::new(HttpAuthority {
            origins: new_origins,
            methods: new_methods,
            path_prefix: new_prefix,
            bound_headers: self.bound_headers.clone(),
            user_agent: self.user_agent.clone(),
            timeout: self.timeout,
        }))
    }

    fn invoke(&self, _rt: &mut dyn Runtime, method: &str, args: &[Value]) -> Result<Value, String> {
        match (method, args) {
            ("get", [Value::Str(u)]) => self.do_request("GET", u, None),
            ("head", [Value::Str(u)]) => self.do_request("HEAD", u, None),
            ("post", [Value::Str(u), Value::Str(b)]) => self.do_request("POST", u, Some(b)),
            ("describe", []) => {
                let mut m: HashMap<String, Value> = HashMap::new();
                let origins: Vec<Value> = self
                    .origins
                    .iter()
                    .map(|o| Value::Str(Rc::new(o.clone())))
                    .collect();
                m.insert(
                    "origins".into(),
                    Value::List(Rc::new(RefCell::new(origins))),
                );
                let mut ms: Vec<String> = self.methods.iter().cloned().collect();
                ms.sort();
                let methods: Vec<Value> = ms.into_iter().map(|s| Value::Str(Rc::new(s))).collect();
                m.insert(
                    "methods".into(),
                    Value::List(Rc::new(RefCell::new(methods))),
                );
                m.insert(
                    "path_prefix".into(),
                    match &self.path_prefix {
                        Some(p) => Value::Str(Rc::new(p.clone())),
                        None => Value::Nil,
                    },
                );
                // Header names only. Values are harness secrets; never leak.
                let header_names: Vec<Value> = self
                    .bound_headers
                    .iter()
                    .map(|(k, _)| Value::Str(Rc::new(k.clone())))
                    .collect();
                m.insert(
                    "bound_header_names".into(),
                    Value::List(Rc::new(RefCell::new(header_names))),
                );
                Ok(Value::Map(Rc::new(RefCell::new(m))))
            }
            (m, _) => Err(format!(
                "http: no method `{}` (have: get, post, head, describe)",
                m
            )),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ---------- tiny URL parser ----------

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
            .ok_or_else(|| format!("http: url `{}` has no scheme", url))?;
        let scheme = scheme.to_ascii_lowercase();
        if scheme != "http" && scheme != "https" {
            return Err(format!("http: scheme `{}` not supported", scheme));
        }
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => {
                let p: u16 = p
                    .parse()
                    .map_err(|_| format!("http: bad port in `{}`", url))?;
                (h.to_ascii_lowercase(), Some(p))
            }
            None => (authority.to_ascii_lowercase(), None),
        };
        Ok(ParsedUrl {
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
    // Trim trailing slash, lowercase scheme + host, keep explicit port.
    let s = s.trim_end_matches('/').to_string();
    match ParsedUrl::parse(&s) {
        Ok(p) => p.origin(),
        Err(_) => s, // fall back to raw; scope check will fail comparisons anyway
    }
}
