//! Language Server Protocol server for Kiln.
//!
//! A minimal LSP implementation: lifecycle, full-text sync, and diagnostics
//! published from lex / parse errors. No hover, completion, or go-to yet —
//! those land once we have real source spans on AST nodes.
//!
//! Speaks JSON-RPC 2.0 over stdio with LSP's Content-Length framing.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn run() -> io::Result<()> {
    let mut docs: HashMap<String, String> = HashMap::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    loop {
        let msg = match read_message(&mut stdin)? {
            Some(m) => m,
            None => return Ok(()), // client closed pipe
        };

        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        if id.is_some() {
            handle_request(id.unwrap(), &method, &params)?;
        } else {
            match method.as_str() {
                "exit" => return Ok(()),
                _ => handle_notification(&method, &params, &mut docs)?,
            }
        }
    }
}

fn handle_request(id: Value, method: &str, _params: &Value) -> io::Result<()> {
    match method {
        "initialize" => {
            let result = json!({
                "capabilities": {
                    "textDocumentSync": 1,
                },
                "serverInfo": {
                    "name": "kiln-lsp",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            });
            send_response(id, Some(result), None)
        }
        "shutdown" => send_response(id, Some(Value::Null), None),
        _ => send_response(id, None, Some(json!({
            "code": -32601,
            "message": format!("method not found: {}", method),
        }))),
    }
}

fn handle_notification(
    method: &str,
    params: &Value,
    docs: &mut HashMap<String, String>,
) -> io::Result<()> {
    match method {
        "initialized" => Ok(()),
        "textDocument/didOpen" => {
            let uri = params.pointer("/textDocument/uri")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            let text = params.pointer("/textDocument/text")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            docs.insert(uri.clone(), text.clone());
            publish_diagnostics(&uri, &text)
        }
        "textDocument/didChange" => {
            let uri = params.pointer("/textDocument/uri")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Some(changes) = params.pointer("/contentChanges").and_then(|v| v.as_array()) {
                if let Some(last) = changes.last() {
                    if let Some(text) = last.get("text").and_then(|v| v.as_str()) {
                        docs.insert(uri.clone(), text.to_string());
                        return publish_diagnostics(&uri, text);
                    }
                }
            }
            Ok(())
        }
        "textDocument/didSave" => {
            let uri = params.pointer("/textDocument/uri")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            if let Some(text) = docs.get(&uri).cloned() {
                return publish_diagnostics(&uri, &text);
            }
            Ok(())
        }
        "textDocument/didClose" => {
            let uri = params.pointer("/textDocument/uri")
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            docs.remove(&uri);
            send_notification("textDocument/publishDiagnostics", json!({
                "uri": uri,
                "diagnostics": [],
            }))
        }
        _ => Ok(()),
    }
}

// ---------- diagnostics ----------

fn publish_diagnostics(uri: &str, text: &str) -> io::Result<()> {
    let diagnostics = compute_diagnostics(text);
    send_notification("textDocument/publishDiagnostics", json!({
        "uri": uri,
        "diagnostics": diagnostics,
    }))
}

fn compute_diagnostics(text: &str) -> Value {
    let mut diags = Vec::new();
    match Lexer::new(text).tokenize() {
        Err(e) => diags.push(diag(e.line, e.col, "lex error", &e.msg)),
        Ok(tokens) => {
            if let Err(e) = Parser::new(tokens).parse_program() {
                diags.push(diag(e.line, e.col, "parse error", &e.msg));
            }
        }
    }
    Value::Array(diags)
}

fn diag(line: usize, col: usize, kind: &str, msg: &str) -> Value {
    // LSP uses zero-indexed line + character offsets; Kiln lexer/parser use
    // 1-indexed so subtract 1 (saturating for robustness).
    let l = line.saturating_sub(1) as u64;
    let c = col.saturating_sub(1) as u64;
    json!({
        "range": {
            "start": {"line": l, "character": c},
            "end":   {"line": l, "character": c + 1},
        },
        "severity": 1, // Error
        "source": "kiln",
        "message": format!("{}: {}", kind, msg),
    })
}

// ---------- JSON-RPC framing ----------

fn read_message(stdin: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        let n = stdin.read_line(&mut line)?;
        if n == 0 { return Ok(None); }
        let line = line.trim_end_matches(|c| c == '\r' || c == '\n');
        if line.is_empty() { break; }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }
    let mut buf = vec![0u8; content_length];
    stdin.read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf).ok())
}

fn send_message(msg: Value) -> io::Result<()> {
    let body = serde_json::to_string(&msg).unwrap();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    out.flush()
}

fn send_response(id: Value, result: Option<Value>, error: Option<Value>) -> io::Result<()> {
    let mut msg = json!({"jsonrpc": "2.0", "id": id});
    if let Some(r) = result { msg["result"] = r; }
    if let Some(e) = error { msg["error"] = e; }
    send_message(msg)
}

fn send_notification(method: &str, params: Value) -> io::Result<()> {
    send_message(json!({"jsonrpc": "2.0", "method": method, "params": params}))
}
