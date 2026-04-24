//! Minimal blocking HTTP/1.1 server and client.
//!
//! Scripts call `http_serve(port, handler)`; this module owns the accept loop
//! and the wire protocol. For each connection we parse a request into a
//! TetherScript Value (a Map with `method`, `path`, `query`, `headers`, `body`),
//! hand it to `handler` via the Runtime bridge, then serialize whatever the
//! handler returns back onto the socket.
//!
//! Scripts can also call `http_get(url)`, `http_head(url)`, `http_post(url, body)`,
//! or `http_request(method, url[, body[, headers]])`. The client uses only
//! `std::net::TcpStream`, so it intentionally supports plain `http://` URLs
//! only. `https://` needs TLS, which is outside the Rust standard library.
//!
//! No Tokio, no threads, no keep-alive. One request per connection, strict
//! request line + Content-Length body. Enough to build real handlers in the
//! language; not enough to put on the open internet.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::time::Duration;

use crate::value::{ResultValue, Runtime, Value};

type HttpResponseParts = (u16, HashMap<String, String>, Vec<u8>);
const MAX_START_LINE_BYTES: usize = 8 * 1024;
const MAX_HEADER_LINE_BYTES: usize = 8 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 1024 * 1024;
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
const CLIENT_USER_AGENT: &str = "tetherscript/0.1 std-http";

pub fn get(url: &Value) -> Value {
    result_value(
        string_arg(url, "http_get: url").and_then(|url| client_request("GET", &url, None, &[])),
    )
}

pub fn head(url: &Value) -> Value {
    result_value(
        string_arg(url, "http_head: url").and_then(|url| client_request("HEAD", &url, None, &[])),
    )
}

pub fn post(url: &Value, body: &Value) -> Value {
    let url = string_arg(url, "http_post: url");
    let body = string_arg(body, "http_post: body");
    let result =
        url.and_then(|url| body.and_then(|body| client_request("POST", &url, Some(&body), &[])));
    result_value(result)
}

pub fn request(args: &[Value]) -> Value {
    result_value(request_inner(args))
}

fn request_inner(args: &[Value]) -> Result<Value, String> {
    if !(2..=4).contains(&args.len()) {
        return Err("http_request expects method, url[, body[, headers]]".into());
    }

    let method = method_arg(&args[0])?;
    let url = string_arg(&args[1], "http_request: url")?;
    let body = match args.get(2) {
        Some(Value::Nil) | None => None,
        Some(value) => Some(string_arg(value, "http_request: body")?),
    };
    let headers = match args.get(3) {
        Some(value) => headers_arg(value)?,
        None => Vec::new(),
    };

    client_request(&method, &url, body.as_deref(), &headers)
}

fn result_value(result: Result<Value, String>) -> Value {
    Value::Result(Rc::new(match result {
        Ok(value) => ResultValue::Ok(value),
        Err(error) => ResultValue::Err(error),
    }))
}

fn client_request(
    method: &str,
    raw_url: &str,
    body: Option<&str>,
    headers: &[(String, String)],
) -> Result<Value, String> {
    let url = ParsedHttpUrl::parse(raw_url)?;
    let body_bytes = body.map(str::as_bytes).unwrap_or(&[]);
    if body_bytes.len() > MAX_BODY_BYTES {
        return Err(format!(
            "http_request: body exceeds {} bytes",
            MAX_BODY_BYTES
        ));
    }

    let mut stream = TcpStream::connect((url.host.as_str(), url.port)).map_err(|e| {
        format!(
            "http_request: connect to {}:{} failed: {}",
            url.host, url.port, e
        )
    })?;
    stream
        .set_read_timeout(Some(CLIENT_TIMEOUT))
        .map_err(|e| format!("http_request: set read timeout failed: {}", e))?;
    stream
        .set_write_timeout(Some(CLIENT_TIMEOUT))
        .map_err(|e| format!("http_request: set write timeout failed: {}", e))?;

    write!(
        stream,
        "{} {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: {}\r\nAccept: */*\r\nConnection: close\r\n",
        method, url.target, url.host_header, CLIENT_USER_AGENT
    )
    .map_err(|e| format!("http_request: write request failed: {}", e))?;

    for (name, value) in headers {
        write!(stream, "{}: {}\r\n", name, value)
            .map_err(|e| format!("http_request: write header failed: {}", e))?;
    }

    if !body_bytes.is_empty() {
        write!(stream, "Content-Length: {}\r\n", body_bytes.len())
            .map_err(|e| format!("http_request: write content-length failed: {}", e))?;
    }

    stream
        .write_all(b"\r\n")
        .and_then(|_| stream.write_all(body_bytes))
        .and_then(|_| stream.flush())
        .map_err(|e| format!("http_request: write body failed: {}", e))?;

    let mut reader = BufReader::new(stream);
    read_client_response(&mut reader, method)
}

fn read_client_response(reader: &mut BufReader<TcpStream>, method: &str) -> Result<Value, String> {
    let line = read_line_limited(reader, MAX_START_LINE_BYTES, "response status-line")?
        .ok_or_else(|| "http_request: empty response".to_string())?;
    let status_line = line.trim_end_matches(['\r', '\n']);
    let mut parts = status_line.splitn(3, ' ');
    let version = parts.next().unwrap_or_default();
    if !version.starts_with("HTTP/") {
        return Err(format!(
            "http_request: invalid response status-line {}",
            status_line
        ));
    }
    let status: u16 = parts
        .next()
        .ok_or_else(|| "http_request: missing response status".to_string())?
        .parse()
        .map_err(|_| format!("http_request: invalid response status {}", status_line))?;
    let reason = parts.next().unwrap_or_default().to_string();

    let headers = read_response_headers(reader)?;
    let body = if method == "HEAD" {
        Vec::new()
    } else if headers
        .get("transfer-encoding")
        .map(|value| has_token(value, "chunked"))
        .unwrap_or(false)
    {
        read_chunked_body(reader)?
    } else if let Some(content_length) = headers.get("content-length") {
        read_content_length_body(reader, content_length)?
    } else {
        read_close_delimited_body(reader)?
    };

    Ok(client_response_value(status, reason, headers, body))
}

fn read_response_headers(
    reader: &mut BufReader<TcpStream>,
) -> Result<HashMap<String, String>, String> {
    let mut headers = HashMap::new();
    let mut header_bytes = 0;
    loop {
        let Some(line) = read_line_limited(reader, MAX_HEADER_LINE_BYTES, "response header")?
        else {
            break;
        };
        header_bytes += line.len();
        if header_bytes > MAX_HEADER_BYTES {
            return Err(format!(
                "http_request: response headers exceed {} bytes",
                MAX_HEADER_BYTES
            ));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Ok(headers)
}

fn read_content_length_body(
    reader: &mut BufReader<TcpStream>,
    content_length: &str,
) -> Result<Vec<u8>, String> {
    let len: usize = content_length
        .parse()
        .map_err(|_| format!("http_request: invalid content-length {}", content_length))?;
    if len > MAX_BODY_BYTES {
        return Err(format!(
            "http_request: content-length {} exceeds {} bytes",
            len, MAX_BODY_BYTES
        ));
    }

    let mut body = vec![0; len];
    if len > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|e| format!("http_request: read body failed: {}", e))?;
    }
    Ok(body)
}

fn read_close_delimited_body(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    reader
        .take((MAX_BODY_BYTES + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|e| format!("http_request: read body failed: {}", e))?;
    if body.len() > MAX_BODY_BYTES {
        return Err(format!(
            "http_request: response body exceeds {} bytes",
            MAX_BODY_BYTES
        ));
    }
    Ok(body)
}

fn read_chunked_body(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    loop {
        let line = read_line_limited(reader, MAX_HEADER_LINE_BYTES, "chunk size")?
            .ok_or_else(|| "http_request: unexpected EOF reading chunk size".to_string())?;
        let size_text = line
            .trim_end_matches(['\r', '\n'])
            .split(';')
            .next()
            .unwrap_or_default()
            .trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| format!("http_request: invalid chunk size {}", size_text))?;

        if size == 0 {
            drain_trailing_headers(reader)?;
            break;
        }
        if body.len() + size > MAX_BODY_BYTES {
            return Err(format!(
                "http_request: response body exceeds {} bytes",
                MAX_BODY_BYTES
            ));
        }

        let start = body.len();
        body.resize(start + size, 0);
        reader
            .read_exact(&mut body[start..])
            .map_err(|e| format!("http_request: read chunk failed: {}", e))?;

        let mut crlf = [0; 2];
        reader
            .read_exact(&mut crlf)
            .map_err(|e| format!("http_request: read chunk terminator failed: {}", e))?;
        if crlf != *b"\r\n" {
            return Err("http_request: invalid chunk terminator".into());
        }
    }
    Ok(body)
}

fn drain_trailing_headers(reader: &mut BufReader<TcpStream>) -> Result<(), String> {
    let mut bytes = 0;
    loop {
        let Some(line) = read_line_limited(reader, MAX_HEADER_LINE_BYTES, "trailer header")? else {
            return Ok(());
        };
        bytes += line.len();
        if bytes > MAX_HEADER_BYTES {
            return Err(format!(
                "http_request: trailer headers exceed {} bytes",
                MAX_HEADER_BYTES
            ));
        }
        if line.trim_end_matches(['\r', '\n']).is_empty() {
            return Ok(());
        }
    }
}

fn client_response_value(
    status: u16,
    reason: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
) -> Value {
    let headers = headers
        .into_iter()
        .map(|(name, value)| (name, Value::Str(Rc::new(value))))
        .collect();
    let mut response = HashMap::new();
    response.insert("status".into(), Value::Int(status as i64));
    response.insert("ok".into(), Value::Bool((200..300).contains(&status)));
    response.insert("reason".into(), Value::Str(Rc::new(reason)));
    response.insert("headers".into(), Value::Map(Rc::new(RefCell::new(headers))));
    response.insert(
        "body".into(),
        Value::Str(Rc::new(String::from_utf8_lossy(&body).into_owned())),
    );
    Value::Map(Rc::new(RefCell::new(response)))
}

fn string_arg(value: &Value, label: &str) -> Result<String, String> {
    match value {
        Value::Str(value) => Ok((**value).clone()),
        other => Err(format!("{} must be str, got {}", label, other.type_name())),
    }
}

fn method_arg(value: &Value) -> Result<String, String> {
    let method = string_arg(value, "http_request: method")?.to_ascii_uppercase();
    if method.is_empty() || !method.bytes().all(is_http_token_byte) {
        return Err(format!("http_request: invalid method {}", method));
    }
    Ok(method)
}

fn headers_arg(value: &Value) -> Result<Vec<(String, String)>, String> {
    let map = match value {
        Value::Map(map) => map.borrow(),
        other => {
            return Err(format!(
                "http_request: headers must be map, got {}",
                other.type_name()
            ))
        }
    };

    let mut headers = Vec::new();
    for (name, value) in map.iter() {
        let name_lower = name.to_ascii_lowercase();
        if matches!(
            name_lower.as_str(),
            "host" | "content-length" | "connection" | "transfer-encoding"
        ) {
            return Err(format!(
                "http_request: header {} is managed by TetherScript",
                name
            ));
        }
        if name.is_empty() || !name.bytes().all(is_http_token_byte) {
            return Err(format!("http_request: invalid header name {}", name));
        }
        let value = match value {
            Value::Str(value) => (**value).clone(),
            other => other.to_string(),
        };
        if value.contains('\r') || value.contains('\n') {
            return Err(format!(
                "http_request: header {} must not contain CR or LF",
                name
            ));
        }
        headers.push((name.clone(), value));
    }
    Ok(headers)
}

fn is_http_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn has_token(value: &str, token: &str) -> bool {
    value
        .split(',')
        .any(|part| part.trim().eq_ignore_ascii_case(token))
}

#[derive(Debug, PartialEq)]
struct ParsedHttpUrl {
    host: String,
    port: u16,
    host_header: String,
    target: String,
}

impl ParsedHttpUrl {
    fn parse(input: &str) -> Result<Self, String> {
        if input.starts_with("https://") {
            return Err(
                "http_request: https:// requires TLS; std-only TetherScript supports http://"
                    .into(),
            );
        }
        let rest = input
            .strip_prefix("http://")
            .ok_or_else(|| "http_request: URL must start with http://".to_string())?;
        if rest.is_empty() {
            return Err("http_request: missing URL authority".into());
        }
        if rest.contains('#') {
            return Err("http_request: URL fragments are not sent over HTTP".into());
        }

        let split_at = rest
            .char_indices()
            .find_map(|(idx, ch)| matches!(ch, '/' | '?').then_some(idx))
            .unwrap_or(rest.len());
        let authority = &rest[..split_at];
        let suffix = &rest[split_at..];
        if authority.is_empty() {
            return Err("http_request: missing URL host".into());
        }
        if authority.contains('@') {
            return Err("http_request: userinfo in URLs is not supported".into());
        }

        let (host, port, host_header) = parse_authority(authority)?;
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

fn parse_authority(authority: &str) -> Result<(String, u16, String), String> {
    if authority.starts_with('[') {
        let end = authority
            .find(']')
            .ok_or_else(|| "http_request: invalid bracketed IPv6 host".to_string())?;
        let host = authority[1..end].to_string();
        if host.is_empty() {
            return Err("http_request: empty IPv6 host".into());
        }
        let rest = &authority[end + 1..];
        let port = if rest.is_empty() {
            80
        } else {
            let port = rest
                .strip_prefix(':')
                .ok_or_else(|| "http_request: invalid authority after IPv6 host".to_string())?;
            parse_port(port)?
        };
        let host_header = if port == 80 {
            format!("[{}]", host)
        } else {
            format!("[{}]:{}", host, port)
        };
        return Ok((host, port, host_header));
    }

    if authority.matches(':').count() > 1 {
        return Err("http_request: IPv6 hosts must be bracketed".into());
    }
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host.to_string(), parse_port(port)?),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return Err("http_request: missing URL host".into());
    }
    if host.chars().any(char::is_whitespace) {
        return Err("http_request: URL host must not contain whitespace".into());
    }
    let host_header = if port == 80 {
        host.clone()
    } else {
        format!("{}:{}", host, port)
    };
    Ok((host, port, host_header))
}

fn parse_port(port: &str) -> Result<u16, String> {
    if port.is_empty() {
        return Err("http_request: empty URL port".into());
    }
    let port: u16 = port
        .parse()
        .map_err(|_| format!("http_request: invalid URL port {}", port))?;
    if port == 0 {
        return Err("http_request: URL port must be greater than zero".into());
    }
    Ok(port)
}

pub fn serve(rt: &mut dyn Runtime, port: &Value, handler: &Value) -> Result<Value, String> {
    let port = match port {
        Value::Int(n) if *n > 0 && *n <= 65535 => *n as u16,
        Value::Int(n) => return Err(format!("http_serve: port {} out of range", n)),
        other => {
            return Err(format!(
                "http_serve: port must be int, got {}",
                other.type_name()
            ))
        }
    };

    if !matches!(handler, Value::Fn(_) | Value::VmFn(_) | Value::Native(_)) {
        return Err(format!(
            "http_serve: handler must be a function, got {}",
            handler.type_name()
        ));
    }

    // Bind all interfaces so containerized / port-forwarded deployments
    // actually receive traffic. Developers running locally can still hit
    // http://127.0.0.1:PORT.
    let addr = ("0.0.0.0", port);
    let listener = TcpListener::bind(addr)
        .map_err(|e| format!("http_serve: bind to 0.0.0.0:{} failed: {}", port, e))?;
    eprintln!(
        "tetherscript http: listening on http://0.0.0.0:{} (try http://localhost:{})",
        port, port
    );

    for conn in listener.incoming() {
        let stream = match conn {
            Ok(s) => s,
            Err(e) => {
                eprintln!("tetherscript http: accept error: {}", e);
                continue;
            }
        };
        if let Err(e) = handle_one(rt, handler, stream) {
            eprintln!("tetherscript http: {}", e);
        }
    }
    Ok(Value::Nil)
}

fn handle_one(rt: &mut dyn Runtime, handler: &Value, mut stream: TcpStream) -> Result<(), String> {
    // Read the head (request line + headers) — bounded to keep a malformed
    // client from eating all our memory.
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| format!("clone stream: {}", e))?,
    );

    let request = match parse_request(&mut reader) {
        Ok(r) => r,
        Err(e) => {
            let _ = write_simple(
                &mut stream,
                400,
                "Bad Request",
                "text/plain; charset=utf-8",
                e.as_bytes(),
            );
            return Err(format!("parse: {}", e));
        }
    };

    let response = match rt.invoke(handler, &[request]) {
        Ok(v) => v,
        Err(e) => {
            let body = format!("handler error: {}", e);
            let _ = write_simple(
                &mut stream,
                500,
                "Internal Server Error",
                "text/plain; charset=utf-8",
                body.as_bytes(),
            );
            return Err(format!("handler: {}", e));
        }
    };

    write_response(&mut stream, &response)
}

// ---------- request parsing ----------

fn parse_request(reader: &mut BufReader<TcpStream>) -> Result<Value, String> {
    let line = read_line_limited(reader, MAX_START_LINE_BYTES, "start-line")?
        .ok_or_else(|| "empty request".to_string())?;
    let start = line.trim_end_matches(['\r', '\n']).to_string();
    let mut parts = start.splitn(3, ' ');
    let method = parts.next().ok_or("missing method")?.to_string();
    let target = parts.next().ok_or("missing target")?.to_string();
    let _version = parts.next().unwrap_or("HTTP/1.1");

    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target, String::new()),
    };

    let mut headers: HashMap<String, Value> = HashMap::new();
    let mut content_length: usize = 0;
    let mut header_bytes: usize = 0;
    loop {
        let Some(h) = read_line_limited(reader, MAX_HEADER_LINE_BYTES, "header")? else {
            break;
        };
        header_bytes += h.len();
        if header_bytes > MAX_HEADER_BYTES {
            return Err(format!("headers exceed {} bytes", MAX_HEADER_BYTES));
        }
        let trimmed = h.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            if name == "content-length" {
                content_length = value.parse().unwrap_or(0);
                if content_length > MAX_BODY_BYTES {
                    return Err(format!(
                        "content-length {} exceeds {} bytes",
                        content_length, MAX_BODY_BYTES
                    ));
                }
            }
            headers.insert(name, Value::Str(Rc::new(value)));
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|e| format!("read body: {}", e))?;
    }
    let body_str = String::from_utf8_lossy(&body).into_owned();

    let mut map: HashMap<String, Value> = HashMap::new();
    map.insert("method".into(), Value::Str(Rc::new(method)));
    map.insert("path".into(), Value::Str(Rc::new(path)));
    map.insert("query".into(), Value::Str(Rc::new(query)));
    map.insert("headers".into(), Value::Map(Rc::new(RefCell::new(headers))));
    map.insert("body".into(), Value::Str(Rc::new(body_str)));
    Ok(Value::Map(Rc::new(RefCell::new(map))))
}

fn read_line_limited(
    reader: &mut BufReader<TcpStream>,
    limit: usize,
    label: &str,
) -> Result<Option<String>, String> {
    let mut out = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|e| format!("read {label}: {e}"))?;
        if available.is_empty() {
            if out.is_empty() {
                return Ok(None);
            }
            break;
        }

        let take_len = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |pos| pos + 1);
        if out.len() + take_len > limit {
            return Err(format!("{label} exceeds {limit} bytes"));
        }
        let has_newline = available[..take_len].last() == Some(&b'\n');
        out.extend_from_slice(&available[..take_len]);
        reader.consume(take_len);
        if has_newline {
            break;
        }
    }

    String::from_utf8(out)
        .map(Some)
        .map_err(|e| format!("read {label}: invalid UTF-8: {e}"))
}

// ---------- response serialization ----------

fn write_response(stream: &mut TcpStream, resp: &Value) -> Result<(), String> {
    let (status, headers, body) = extract_response(resp)?;
    let reason = reason_phrase(status);
    let content_type = headers
        .get("content-type")
        .cloned()
        .unwrap_or_else(|| "text/plain; charset=utf-8".to_string());

    write!(stream, "HTTP/1.1 {} {}\r\n", status, reason)
        .map_err(|e| format!("write status: {}", e))?;
    write!(stream, "Content-Length: {}\r\n", body.len()).map_err(|e| format!("write cl: {}", e))?;
    write!(stream, "Content-Type: {}\r\n", content_type).map_err(|e| format!("write ct: {}", e))?;
    write!(stream, "Connection: close\r\n").map_err(|e| format!("write conn: {}", e))?;
    for (k, v) in &headers {
        if k == "content-length" || k == "content-type" || k == "connection" {
            continue;
        }
        write!(stream, "{}: {}\r\n", k, v).map_err(|e| format!("write header: {}", e))?;
    }
    stream
        .write_all(b"\r\n")
        .map_err(|e| format!("write sep: {}", e))?;
    stream
        .write_all(&body)
        .map_err(|e| format!("write body: {}", e))?;
    stream.flush().map_err(|e| format!("flush: {}", e))?;
    Ok(())
}

fn write_simple(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    write!(stream, "HTTP/1.1 {} {}\r\n", status, reason).map_err(|e| e.to_string())?;
    write!(stream, "Content-Length: {}\r\n", body.len()).map_err(|e| e.to_string())?;
    write!(stream, "Content-Type: {}\r\n", content_type).map_err(|e| e.to_string())?;
    write!(stream, "Connection: close\r\n\r\n").map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())
}

fn extract_response(resp: &Value) -> Result<HttpResponseParts, String> {
    match resp {
        Value::Str(s) => Ok((200, HashMap::new(), s.as_bytes().to_vec())),
        Value::Nil => Ok((204, HashMap::new(), Vec::new())),
        Value::Map(m) => {
            let m = m.borrow();
            let status = match m.get("status") {
                Some(Value::Int(n)) => *n as u16,
                Some(other) => {
                    return Err(format!(
                        "response.status must be int, got {}",
                        other.type_name()
                    ))
                }
                None => 200,
            };
            let body = match m.get("body") {
                Some(Value::Str(s)) => s.as_bytes().to_vec(),
                Some(Value::Nil) | None => Vec::new(),
                Some(other) => other.to_string().into_bytes(),
            };
            let mut headers = HashMap::new();
            if let Some(Value::Map(h)) = m.get("headers") {
                for (k, v) in h.borrow().iter() {
                    headers.insert(k.to_ascii_lowercase(), v.to_string());
                }
            }
            Ok((status, headers, body))
        }
        other => Err(format!(
            "handler must return a str or map, got {}",
            other.type_name()
        )),
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::interp::Interpreter;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::vm::VM;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    fn spawn_once(response: Vec<u8>) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            loop {
                let mut line = String::new();
                let n = reader.read_line(&mut line).unwrap();
                if n == 0 {
                    break;
                }
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            stream.write_all(&response).unwrap();
            stream.flush().unwrap();
            request
        });
        (format!("http://{}", addr), handle)
    }

    #[test]
    fn parses_plain_http_urls() {
        assert_eq!(
            ParsedHttpUrl::parse("http://example.com:8080/path?q=1").unwrap(),
            ParsedHttpUrl {
                host: "example.com".into(),
                port: 8080,
                host_header: "example.com:8080".into(),
                target: "/path?q=1".into(),
            }
        );
        assert_eq!(
            ParsedHttpUrl::parse("http://example.com?q=1")
                .unwrap()
                .target,
            "/?q=1"
        );
    }

    #[test]
    fn rejects_https_without_tls() {
        let result = get(&Value::Str(Rc::new("https://example.com/".into())));
        match result {
            Value::Result(result) => match result.as_ref() {
                ResultValue::Err(error) => assert!(error.contains("https:// requires TLS")),
                other => panic!(
                    "expected Err, got {:?}",
                    Value::Result(Rc::new(other.clone()))
                ),
            },
            other => panic!("expected Result, got {:?}", other),
        }
    }

    #[test]
    fn get_fetches_plain_http() {
        let (base, handle) = spawn_once(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 18\r\n\r\nhello tetherscript"
                .to_vec(),
        );
        let response = client_request("GET", &format!("{}/demo?x=1", base), None, &[]).unwrap();
        match response {
            Value::Map(map) => {
                let map = map.borrow();
                assert_eq!(map.get("status"), Some(&Value::Int(200)));
                assert_eq!(map.get("ok"), Some(&Value::Bool(true)));
                assert_eq!(
                    map.get("body"),
                    Some(&Value::Str(Rc::new("hello tetherscript".into())))
                );
            }
            other => panic!("expected response map, got {:?}", other),
        }

        let request = handle.join().unwrap();
        assert!(request.starts_with("GET /demo?x=1 HTTP/1.1\r\n"));
        assert!(request.contains("\r\nHost: 127.0.0.1:"));
    }

    #[test]
    fn chunked_response_is_decoded() {
        let (base, handle) = spawn_once(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n1\r\n!\r\n0\r\n\r\n"
                .to_vec(),
        );
        let response = client_request("GET", &base, None, &[]).unwrap();
        match response {
            Value::Map(map) => {
                assert_eq!(
                    map.borrow().get("body"),
                    Some(&Value::Str(Rc::new("hello!".into())))
                );
            }
            other => panic!("expected response map, got {:?}", other),
        }
        handle.join().unwrap();
    }

    #[test]
    fn interpreter_can_call_http_get_builtin() {
        let (base, handle) =
            spawn_once(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nfrom builtin".to_vec());
        let source = format!(
            r#"
let resp = http_get("{}").unwrap()
resp.body
"#,
            base
        );
        let tokens = Lexer::new(&source).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let mut interp = Interpreter::new();
        let result = interp.run_repl(&program).unwrap();

        assert_eq!(result, Value::Str(Rc::new("from builtin".into())));
        handle.join().unwrap();
    }

    #[test]
    fn vm_can_call_http_get_builtin() {
        let (base, handle) =
            spawn_once(b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nfrom vm".to_vec());
        let source = format!(
            r#"
fn main() {{
    let resp = http_get("{}").unwrap()
    if resp.body != "from vm" {{
        panic "bad body"
    }}
}}
"#,
            base
        );
        let tokens = Lexer::new(&source).tokenize().unwrap();
        let program = Parser::new(tokens).parse_program().unwrap();
        let chunk = Compiler::compile_program(&program);
        let mut vm = VM::new();

        vm.run(chunk).unwrap();
        handle.join().unwrap();
    }
}
