//! Unit tests for the pump loop: flushing and termination.
//!
//! Bound enforcement has its own file, [`super::starve_tests`], because it is the
//! rule protecting the single-threaded accept loop and deserves to be found
//! without reading past the framing assertions.

use std::rc::Rc;

use crate::value::{ResultValue, Value};

use super::support::ScriptedRuntime;
use super::values::{map, native, str_value};
use super::{run_pump, shape, StopReason, StreamSpec};

/// Parse a streaming response from `entries`, panicking on invalid input.
pub(super) fn spec_of(entries: &[(&str, Value)]) -> StreamSpec {
    shape::parse(&map(entries)).expect("spec should parse")
}

#[test]
fn the_pump_flushes_each_frame_then_terminates_the_body() {
    let second = Value::Result(Rc::new(ResultValue::Ok(str_value("data: two\n\n"))));
    let frames = vec![Ok(str_value("data: one\n\n")), Ok(second)];
    let mut runtime = ScriptedRuntime::new(frames);
    let spec = spec_of(&[("stream", native()), ("chunked", Value::Bool(true))]);
    let mut sink: Vec<u8> = Vec::new();
    let outcome = run_pump(&mut runtime, &mut sink, &spec);
    assert_eq!(outcome.events, 2);
    assert_eq!(outcome.stop, StopReason::Exhausted);
    let expected = b"b\r\ndata: one\n\n\r\nb\r\ndata: two\n\n\r\n0\r\n\r\n".to_vec();
    assert_eq!(sink, expected);
}

#[test]
fn close_coding_writes_payloads_verbatim_with_no_terminator() {
    let frames = vec![Ok(str_value(": tick\n")), Ok(str_value("retry: 500\n\n"))];
    let mut runtime = ScriptedRuntime::new(frames);
    let spec = spec_of(&[("stream", native())]);
    let mut sink: Vec<u8> = Vec::new();
    let outcome = run_pump(&mut runtime, &mut sink, &spec);
    assert_eq!(outcome.events, 2);
    assert_eq!(sink, b": tick\nretry: 500\n\n".to_vec());
}

#[test]
fn a_generator_error_ends_the_stream_and_is_reported() {
    let frames = vec![Ok(str_value("data: one\n\n")), Err("boom".to_string())];
    let mut runtime = ScriptedRuntime::new(frames);
    let mut sink: Vec<u8> = Vec::new();
    let spec = spec_of(&[("stream", native())]);
    let outcome = run_pump(&mut runtime, &mut sink, &spec);
    assert_eq!(outcome.events, 1);
    assert_eq!(outcome.stop, StopReason::GeneratorError("boom".to_string()));
}
