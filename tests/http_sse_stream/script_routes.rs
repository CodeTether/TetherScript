//! The routing half of the test program: dispatch, response shapes, and `main`.
//!
//! `streaming` builds the streaming shape; `handle`'s `/health` branch returns an
//! ordinary string response, so both shapes are live on one server.

/// Response builders, the dispatcher, and the entry point.
pub(super) const SOURCE: &str = r#"
fn streaming(generator, chunked, max_events) {
    let resp = map()
    resp.status = 200
    resp.stream = generator
    resp.chunked = chunked
    resp.max_events = max_events
    return resp
}

fn handle(req) {
    let path = req.path
    if path == "/health" { return "ok\n" }
    if path == "/events" { return streaming(ticker(3, 0), false, 100) }
    if path == "/slow" { return streaming(ticker(3, 150), false, 100) }
    if path == "/mixed" { return streaming(mixed(), false, 100) }
    if path == "/multiline" { return streaming(one_multiline(), false, 100) }
    if path == "/chunked" { return streaming(ticker(2, 0), true, 100) }
    if path == "/runaway" { return streaming(runaway(0), false, 4) }
    if path == "/endless" { return streaming(runaway(40), false, 1000) }
    let resp = map()
    resp.status = 404
    resp.body = "no route\n"
    return resp
}

fn port() {
    return parse_int(env_get("RUST_SSE_ADDR").unwrap()).unwrap()
}

fn main() {
    http_serve(port(), handle)
}
"#;
