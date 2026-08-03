//! Unit tests for client-disconnect handling.
//!
//! A closed browser tab is the common case for SSE. These tests use a sink that
//! fails with `BrokenPipe` to prove the pump stops cleanly instead of panicking
//! or spinning: the failure mode being guarded against is one closed tab turning
//! into a busy server thread.

use std::io::{Error, ErrorKind};

use crate::value::Value;

use super::support::{DeadPeer, ScriptedRuntime};
use super::values::{map, native, str_value};
use super::{flush_all, is_disconnect, run_pump, shape, Flow, StopReason};

#[test]
fn the_peer_gone_error_kinds_are_recognised() {
    for kind in [
        ErrorKind::BrokenPipe,
        ErrorKind::ConnectionReset,
        ErrorKind::ConnectionAborted,
        ErrorKind::NotConnected,
        ErrorKind::UnexpectedEof,
    ] {
        assert!(is_disconnect(&Error::new(kind, "gone")), "{kind:?}");
    }
    let other = Error::new(ErrorKind::PermissionDenied, "nope");
    assert!(!is_disconnect(&other), "a real fault is not a disconnect");
}

#[test]
fn a_write_to_a_dead_peer_reports_closed_not_an_error() {
    let mut peer = DeadPeer { ok_writes: 0 };
    assert_eq!(flush_all(&mut peer, b"data: x\n\n").unwrap(), Flow::Closed);
}

#[test]
fn a_disconnect_mid_stream_ends_the_pump_without_looping() {
    // An effectively endless generator plus a peer that dies after two writes.
    let frames = (0..10_000).map(|_| Ok(str_value("data: x\n\n"))).collect();
    let mut runtime = ScriptedRuntime::new(frames);
    let response = map(&[("stream", native()), ("max_events", Value::Int(10_000))]);
    let spec = shape::parse(&response).expect("spec should parse");
    let mut peer = DeadPeer { ok_writes: 2 };
    let outcome = run_pump(&mut runtime, &mut peer, &spec);
    assert_eq!(outcome.stop, StopReason::Disconnected);
    assert_eq!(outcome.events, 2, "only the delivered events count");
    assert_eq!(runtime.calls, 3, "the pump stops at the failed write");
}
