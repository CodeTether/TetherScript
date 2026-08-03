//! The generator half of the test program: closures that produce SSE frames.
//!
//! Each generator takes no arguments and returns a frame or `nil`, which is the
//! contract `src/http_stream_response_pump_payload.rs` documents.

/// Frame helpers and the four generators the routes use.
pub(super) const SOURCE: &str = r#"
fn frame(text) {
    let f = map()
    f.data = text
    return sse_event(f).unwrap()
}

// Yields `count` data events, sleeping `delay` ms before each, then nil.
fn ticker(count, delay) {
    let mut i = 0
    return fn() {
        i = i + 1
        if i > count { return nil }
        if delay > 0 { sleep_ms(delay).unwrap() }
        return frame("tick {i}")
    }
}

// Never returns nil: only the server's bound can end this stream.
fn runaway(delay) {
    return fn() {
        if delay > 0 { sleep_ms(delay).unwrap() }
        return frame("forever")
    }
}

// A comment, then a retry frame, then one event, then end of stream.
fn mixed() {
    let mut step = 0
    return fn() {
        step = step + 1
        if step == 1 { return sse_comment("keep-alive").unwrap() }
        if step == 2 { return sse_retry(2500).unwrap() }
        if step == 3 { return frame("after") }
        return nil
    }
}

// One event whose data spans two lines, to prove per-line `data:` prefixing.
fn one_multiline() {
    let mut sent = false
    return fn() {
        if sent { return nil }
        sent = true
        return frame("first\nsecond")
    }
}
"#;
