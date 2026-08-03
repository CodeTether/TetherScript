//! Owned-resource lifecycle controls, operations, and factories.
//!
//! Ported from `editor/vscode/lib/resource-method-controls.js` and
//! `resource-method-operations.js`. The `resource.*` constructors live in
//! `src/lsp_capabilities_methods_factory.rs`.
//!
//! # Examples
//!
//! ```rust
//! use tetherscript::lsp_capabilities::methods_resource::{CONTROLS, OPERATIONS};
//! assert!(CONTROLS.iter().any(|entry| entry.0 == "close"));
//! assert!(OPERATIONS.iter().any(|entry| entry.0 == "send"));
//! ```

use crate::lsp_capabilities::methods::Method;

/// Lifecycle controls shared by every owned resource.
#[rustfmt::skip]
pub const CONTROLS: &[Method] = &[
    ("cancel", "resource.cancel()", "Cancel work, release the handle, and return a Result."),
    ("clear_deadline", "resource.clear_deadline()", "Remove the resource deadline."),
    ("close", "resource.close()", "Release the owned handle and return a Result."),
    ("deadline_remaining_ms", "resource.deadline_remaining_ms()", "Return remaining deadline time or nil."),
    ("is_cancelled", "resource.is_cancelled()", "Return whether the resource was cancelled."),
    ("is_closed", "resource.is_closed()", "Return whether the resource was closed."),
    ("is_expired", "resource.is_expired()", "Return whether the resource deadline elapsed."),
    ("kind", "resource.kind()", "Return the resource kind name."),
    ("set_deadline", "resource.set_deadline(delay_ms)", "Set a monotonic resource deadline."),
];

/// Per-resource operations.
#[rustfmt::skip]
pub const OPERATIONS: &[Method] = &[
    ("accept", "tcp_listener.accept()", "Accept a nonblocking TCP connection as a Result."),
    ("body", "response_writer.body()", "Return buffered response bytes."),
    ("capacity", "resource.capacity()", "Return a bounded resource capacity."),
    ("complete", "task.complete(value)", "Complete a pending task once."),
    ("flush", "file.flush()", "Flush buffered file data as a Result."),
    ("id", "child_process.id()", "Return the child process ID."),
    ("is_complete", "task.is_complete()", "Return whether a task has completed."),
    ("is_full", "channel.is_full()", "Return whether a channel reached capacity."),
    ("kill", "child_process.kill()", "Terminate a child process as a Result."),
    ("local_addr", "tcp_listener.local_addr()", "Return the listener address as a Result."),
    ("peer_addr", "tcp_stream.peer_addr()", "Return the peer address as a Result."),
    ("port", "tcp_listener.port()", "Return the bound TCP port as a Result."),
    ("read", "resource.read(limit)", "Read available bytes without blocking."),
    ("ready", "timer.ready()", "Return whether a timer is ready."),
    ("recv", "channel.recv()", "Receive a queued value or backpressure Err."),
    ("remaining", "request_body.remaining()", "Return unread request-body bytes."),
    ("remaining_ms", "timer.remaining_ms()", "Return time until the timer is ready."),
    ("reset", "timer.reset(delay_ms)", "Reset a monotonic timer."),
    ("result", "task.result()", "Return task output or a pending backpressure Err."),
    ("send", "channel.send(value)", "Queue a value or return a backpressure Err."),
    ("shutdown", "tcp_stream.shutdown()", "Shut down a TCP stream as a Result."),
    ("try_wait", "child_process.try_wait()", "Poll child completion without blocking."),
    ("wait", "child_process.wait()", "Wait for child completion within its deadline."),
    ("write", "resource.write(value)", "Write within resource capacity as a Result."),
];
