//! Integration tests for the SSE streaming transport (`src/sse/`).
//!
//! The module tree is included by path rather than through `tetherscript::sse`,
//! because exposing it from `lib.rs` is the integrator's one-line change and this
//! file must not depend on it having happened yet. The tree is self-contained — it
//! references nothing outside itself — so this include is exact.
//!
//! Each test names the client-visible failure it prevents. "It looked fine in a
//! capture" is how every one of these bugs survives review.

// Public items reachable only from these tests would otherwise warn as dead code,
// because an integration test sees this tree as a private module of its own crate.
#[allow(dead_code)]
#[path = "../src/sse/stream.rs"]
mod sse;

use std::collections::HashMap;

use crate::sse::backpressure::over_budget;
use crate::sse::keepalive::{is_due, remaining_ms, DEFAULT_INTERVAL_MS};
use crate::sse::last_event_id::{from_map, from_pairs};
use crate::sse::{head, Event, EventStream, SseError, DEFAULT_BOUND};

// ---------------------------------------------------------------------------
// Frame bytes
// ---------------------------------------------------------------------------

/// The blank line is what dispatches the event. Without it the client buffers
/// forever and the page shows nothing, with no error anywhere to explain it.
#[test]
fn single_line_data_event_is_byte_exact_including_blank_line() {
    let mut stream = EventStream::new();
    stream.send_data("hello");
    assert_eq!(stream.as_bytes(), b"data: hello\n\n");
    assert!(stream.as_bytes().ends_with(b"\n\n"), "missing terminator");
}

#[test]
fn event_render_matches_stream_send() {
    let event = Event::data("hello");
    let mut stream = EventStream::new();
    stream.send(&event).unwrap();
    assert_eq!(stream.as_bytes(), event.render().unwrap().as_bytes());
}

/// A raw newline ends the field, so a two-line payload sent as one `data:` line
/// silently truncates at the newline.
#[test]
fn multi_line_payload_becomes_one_data_line_each() {
    let mut stream = EventStream::new();
    stream.send_data("first\nsecond\nthird");
    assert_eq!(
        stream.as_bytes(),
        b"data: first\ndata: second\ndata: third\n\n"
    );
}

#[test]
fn crlf_payload_never_puts_a_carriage_return_on_the_wire() {
    let mut stream = EventStream::new();
    stream.send_data("first\r\nsecond");
    assert_eq!(stream.as_bytes(), b"data: first\ndata: second\n\n");
    assert!(!stream.as_bytes().contains(&b'\r'));
}

#[test]
fn lone_cr_payload_is_split_like_a_newline() {
    let mut stream = EventStream::new();
    stream.send_data("first\rsecond");
    assert_eq!(stream.as_bytes(), b"data: first\ndata: second\n\n");
}

/// A trailing lone CR must not dispatch a second, empty event: exactly one
/// blank-line terminator may appear, and only at the very end.
#[test]
fn trailing_lone_cr_does_not_produce_an_extra_empty_event() {
    let mut stream = EventStream::new();
    stream.send_data("first\r");
    assert_eq!(stream.as_bytes(), b"data: first\ndata: \n\n");
    let text = String::from_utf8(stream.take()).unwrap();
    assert_eq!(text.matches("\n\n").count(), 1, "more than one event");
    assert!(text.ends_with("\n\n"));
}

#[test]
fn empty_payload_is_a_valid_single_empty_data_event() {
    let mut stream = EventStream::new();
    stream.send_data("");
    assert_eq!(stream.as_bytes(), b"data: \n\n");
}

#[test]
fn named_event_puts_the_event_field_before_data() {
    let mut stream = EventStream::new();
    stream.send(&Event::data("payload").name("tick")).unwrap();
    assert_eq!(stream.as_bytes(), b"event: tick\ndata: payload\n\n");
}

#[test]
fn id_is_emitted_after_event_and_before_data() {
    let mut stream = EventStream::new();
    stream.send(&Event::data("payload").id("42")).unwrap();
    assert_eq!(stream.as_bytes(), b"id: 42\ndata: payload\n\n");
}

/// `retry:` is an integer number of milliseconds.
#[test]
fn retry_is_integer_milliseconds() {
    let mut stream = EventStream::new();
    stream.send_retry(3000);
    assert_eq!(stream.as_bytes(), b"retry: 3000\n\n");

    let framed = Event::data("x").retry_ms(1).render().unwrap();
    assert_eq!(framed, "retry: 1\ndata: x\n\n");
}

#[test]
fn all_fields_render_in_fixed_order_with_data_last() {
    let framed = Event::data("body\nmore")
        .name("tick")
        .id("9")
        .retry_ms(500)
        .render()
        .unwrap();
    assert_eq!(
        framed,
        "event: tick\nid: 9\nretry: 500\ndata: body\ndata: more\n\n"
    );
}

/// A comment dispatches nothing, so it carries no blank-line terminator.
#[test]
fn comment_keepalive_is_a_single_unterminated_line() {
    let mut stream = EventStream::new();
    stream.send_keepalive();
    assert_eq!(stream.as_bytes(), b": ping\n");
    assert!(!stream.as_bytes().ends_with(b"\n\n"));

    let mut custom = EventStream::new();
    custom.send_comment("still here").unwrap();
    assert_eq!(custom.as_bytes(), b": still here\n");
}

// ---------------------------------------------------------------------------
// Rejection, not sanitisation
// ---------------------------------------------------------------------------

/// An injected newline in an id lets a caller forge an event boundary, so the id
/// is rejected rather than quietly stripped.
#[test]
fn id_containing_a_newline_is_rejected() {
    let forged = Event::data("x").id("1\n\ndata: forged");
    let err = forged.render().unwrap_err();
    assert_eq!(err, SseError::InvalidId);
    assert!(err.to_string().contains("id"), "error must name the field");
}

#[test]
fn id_containing_cr_or_nul_is_rejected() {
    assert_eq!(
        Event::data("x").id("1\r2").render().unwrap_err(),
        SseError::InvalidId
    );
    assert_eq!(
        Event::data("x").id("1\u{0}2").render().unwrap_err(),
        SseError::InvalidId
    );
}

#[test]
fn multi_line_event_name_and_comment_are_rejected() {
    assert_eq!(
        Event::data("x").name("a\nb").render().unwrap_err(),
        SseError::MultiLineField("event")
    );
    assert_eq!(
        EventStream::new().send_comment("a\nb").unwrap_err(),
        SseError::MultiLineField("comment")
    );
}

/// A rejected event must leave no partial frame behind, or the next good event
/// inherits a forged prefix.
#[test]
fn a_rejected_event_buffers_nothing() {
    let mut stream = EventStream::new();
    assert!(stream.send(&Event::data("x").id("bad\nid")).is_err());
    assert!(stream.is_empty());
    assert_eq!(stream.buffered(), 0);
}

// ---------------------------------------------------------------------------
// Response head
// ---------------------------------------------------------------------------

#[test]
fn head_declares_an_event_stream_and_never_a_content_length() {
    let response_head = head::ok();
    assert!(response_head.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response_head.contains("Content-Type: text/event-stream"));
    assert!(response_head.contains("Cache-Control: no-store\r\n"));
    assert!(response_head.contains("Connection: keep-alive\r\n"));
    assert!(
        !response_head.to_ascii_lowercase().contains("content-length"),
        "a Content-Length truncates a body that never ends: {response_head:?}"
    );
    assert!(response_head.ends_with("\r\n\r\n"), "head needs a blank line");
}

#[test]
fn head_drops_reserved_headers_but_keeps_caller_headers() {
    let response_head = head::render_with(&[
        ("X-Accel-Buffering", "no"),
        ("content-length", "5"),
        ("Content-Length", "5"),
        ("Connection", "close"),
    ]);
    assert!(response_head.contains("X-Accel-Buffering: no\r\n"));
    assert!(!response_head.to_ascii_lowercase().contains("content-length"));
    assert!(!response_head.contains("Connection: close"));
    assert_eq!(response_head.matches("Connection: keep-alive").count(), 1);
}

// ---------------------------------------------------------------------------
// Last-Event-ID resume
// ---------------------------------------------------------------------------

#[test]
fn last_event_id_is_read_case_insensitively() {
    let names = ["Last-Event-ID", "Last-Event-Id", "last-event-id", "LAST-EVENT-ID"];
    for name in names {
        let headers = HashMap::from([(name.to_string(), "42".to_string())]);
        assert_eq!(from_map(&headers), Some("42"), "failed for {name}");
    }
    assert_eq!(from_pairs(&[("LAST-event-id", "7")]), Some("7"));
}

#[test]
fn last_event_id_is_absent_when_missing_or_blank() {
    let empty: HashMap<String, String> = HashMap::new();
    assert_eq!(from_map(&empty), None);

    let other = HashMap::from([("Accept".to_string(), "text/event-stream".to_string())]);
    assert_eq!(from_map(&other), None);

    let blank = HashMap::from([("Last-Event-ID".to_string(), "   ".to_string())]);
    assert_eq!(from_map(&blank), None);

    assert_eq!(from_pairs(&[("Accept", "*/*")]), None);
}

#[test]
fn last_event_id_value_is_trimmed() {
    let headers = HashMap::from([("Last-Event-ID".to_string(), " 42 ".to_string())]);
    assert_eq!(from_map(&headers), Some("42"));
}

// ---------------------------------------------------------------------------
// Keepalive policy
// ---------------------------------------------------------------------------

/// Exactly at the interval a keepalive is due: a proxy with an equal idle timeout
/// otherwise wins the race and closes the connection.
#[test]
fn keepalive_is_due_at_and_past_the_interval_but_not_below() {
    assert!(!is_due(14_999, 0, 15_000), "below the interval");
    assert!(is_due(15_000, 0, 15_000), "exactly at the interval");
    assert!(is_due(15_001, 0, 15_000), "past the interval");
}

#[test]
fn keepalive_measures_from_the_last_write_not_from_zero() {
    assert!(!is_due(20_000, 10_000, 15_000));
    assert!(is_due(25_000, 10_000, 15_000));
}

#[test]
fn keepalive_tolerates_a_clock_that_went_backwards() {
    assert!(!is_due(0, 15_000, 15_000));
    assert_eq!(remaining_ms(0, 15_000, 15_000), 15_000);
}

#[test]
fn keepalive_remaining_reaches_zero_exactly_when_due() {
    assert_eq!(remaining_ms(1_000, 0, 15_000), 14_000);
    assert_eq!(remaining_ms(15_000, 0, 15_000), 0);
    assert_eq!(remaining_ms(99_000, 0, 15_000), 0);
    assert_eq!(DEFAULT_INTERVAL_MS, 15_000);
}

#[test]
fn keepalive_with_a_zero_interval_is_always_due() {
    assert!(is_due(0, 0, 0));
}

// ---------------------------------------------------------------------------
// Backpressure
// ---------------------------------------------------------------------------

/// At the bound is already over budget, so a bound of zero drops immediately.
#[test]
fn backpressure_triggers_at_and_past_the_bound() {
    assert!(!over_budget(0, 64));
    assert!(!over_budget(63, 64));
    assert!(over_budget(64, 64), "at the bound is over budget");
    assert!(over_budget(65, 64));
    assert!(over_budget(0, 0));
}

#[test]
fn stream_reports_drop_once_its_own_bound_is_reached() {
    // "data: x\n\n" is exactly nine bytes.
    let mut stream = EventStream::with_bound(9);
    assert!(!stream.should_drop());
    stream.send_data("x");
    assert_eq!(stream.buffered(), 9);
    assert!(stream.should_drop());
}

#[test]
fn draining_the_buffer_clears_the_drop_condition() {
    let mut stream = EventStream::with_bound(9);
    stream.send_data("x");
    assert!(stream.should_drop());
    let _ = stream.take();
    assert!(stream.is_empty());
    assert!(!stream.should_drop());
}

#[test]
fn default_bound_is_sixty_four_kibibytes() {
    assert_eq!(DEFAULT_BOUND, 64 * 1024);
    assert_eq!(EventStream::new().bound(), DEFAULT_BOUND);
    assert_eq!(EventStream::default().bound(), DEFAULT_BOUND);
}

// ---------------------------------------------------------------------------
// Buffer mechanics
// ---------------------------------------------------------------------------

#[test]
fn frames_accumulate_in_wire_order_until_drained() {
    let mut stream = EventStream::new();
    stream.send_retry(1000);
    stream.send_data("one");
    stream.send_keepalive();
    stream.send(&Event::data("two").name("tick")).unwrap();
    assert_eq!(
        String::from_utf8(stream.take()).unwrap(),
        "retry: 1000\n\ndata: one\n\n: ping\nevent: tick\ndata: two\n\n"
    );
    assert!(stream.is_empty());
}

#[test]
fn push_raw_appends_verbatim() {
    let mut stream = EventStream::new();
    stream.push_raw(b"data: raw\n\n");
    assert_eq!(stream.as_bytes(), b"data: raw\n\n");
    assert_eq!(stream.buffered(), 11);
}