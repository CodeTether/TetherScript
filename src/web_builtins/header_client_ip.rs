//! Client address resolution behind a proxy.
//!
//! # Security: `X-Forwarded-For` is client-controlled
//!
//! Any caller can send `X-Forwarded-For: 1.2.3.4`. Nothing about the header is
//! authenticated, so it is trustworthy **only** when a reverse proxy you control
//! overwrites it on every inbound request. If the header is honored on a
//! directly-exposed listener, a caller can forge its own address and defeat
//! per-IP rate limiting, ban lists, and audit trails — while making the forged
//! value look authoritative in the logs.
//!
//! Prefer `remote_addr` unless the deployment guarantees a rewriting proxy. The
//! reference deployment terminates at an ingress that sets both headers.

use std::collections::HashMap;
use std::rc::Rc;

use super::header_lookup::find;
use crate::value::Value;

/// Resolve the client address, preferring proxy headers.
///
/// # Arguments
///
/// * `headers` — Header map.
/// * `remote_addr` — Peer address of the socket, used when no proxy header applies.
///
/// # Returns
///
/// The leftmost `X-Forwarded-For` entry when present, else `X-Real-IP`, else
/// `remote_addr`. The leftmost entry is the original client because each proxy
/// appends; later entries are the proxies themselves.
pub(super) fn resolve(headers: &HashMap<String, Value>, remote_addr: &str) -> Value {
    if let Some(forwarded) = find(headers, "x-forwarded-for") {
        // A single-hop request has no comma, so `split` still yields the value.
        if let Some(first) = forwarded.split(',').next() {
            let first = first.trim();
            if !first.is_empty() {
                return Value::Str(Rc::new(first.to_string()));
            }
        }
    }
    if let Some(real_ip) = find(headers, "x-real-ip") {
        if !real_ip.is_empty() {
            return Value::Str(Rc::new(real_ip));
        }
    }
    Value::Str(Rc::new(remote_addr.to_string()))
}
