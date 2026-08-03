//! Unit tests for the bounds that protect the single-threaded accept loop.
//!
//! `http_serve` serves one connection at a time, so an unbounded generator does
//! not merely misbehave — it starves every other client, health check included.
//! These tests pin that each cap is enforced *before* the next generator call, so
//! a runaway cannot overshoot it.

use crate::value::Value;

use super::pump_tests::spec_of;
use super::slow::SlowRuntime;
use super::support::ScriptedRuntime;
use super::values::{native, str_value};
use super::{run_pump, StopReason};

#[test]
fn the_event_bound_stops_a_runaway_generator() {
    let frames = (0..50).map(|_| Ok(str_value("data: x\n\n"))).collect();
    let mut runtime = ScriptedRuntime::new(frames);
    let spec = spec_of(&[("stream", native()), ("max_events", Value::Int(3))]);
    let mut sink: Vec<u8> = Vec::new();
    let outcome = run_pump(&mut runtime, &mut sink, &spec);
    assert_eq!(outcome.events, 3, "bound must not be overshot");
    assert_eq!(outcome.stop, StopReason::MaxEvents);
    assert_eq!(runtime.calls, 3, "no generator call past the bound");
    assert_eq!(sink, b"data: x\n\ndata: x\n\ndata: x\n\n".to_vec());
}

#[test]
fn the_duration_bound_stops_a_slow_endless_generator() {
    // A generator that never returns nil, costing 20ms per event, against a 30ms
    // lifetime: the event cap is far away, so only the clock can end this.
    let mut runtime = SlowRuntime::new(str_value("data: tick\n\n"), 20);
    let spec = spec_of(&[
        ("stream", native()),
        ("max_duration_ms", Value::Int(30)),
        ("max_events", Value::Int(100_000)),
    ]);
    let mut sink: Vec<u8> = Vec::new();
    let outcome = run_pump(&mut runtime, &mut sink, &spec);
    assert_eq!(outcome.stop, StopReason::MaxDuration);
    assert!(outcome.events >= 1, "at least one event should go out");
    assert_eq!(
        runtime.calls as u32, outcome.events,
        "every call that produced a frame delivered it"
    );
    assert!(
        outcome.events < 100,
        "the clock, not the event cap, ended it: {}",
        outcome.events
    );
}
