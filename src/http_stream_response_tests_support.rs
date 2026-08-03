//! Fake sinks and a scripted runtime for the streaming-response unit tests.
//!
//! Split from [`super::values`] so value building and fake I/O stay separate.

use std::io::{self, ErrorKind, Write};

use crate::value::{Runtime, Value};

/// A sink that accepts `ok_writes` writes and then reports a dead peer.
///
/// # Examples
///
/// ```text
/// let mut peer = DeadPeer { ok_writes: 2 };  // two events, then BrokenPipe
/// ```
pub(super) struct DeadPeer {
    /// Writes still permitted before the pipe breaks.
    pub(super) ok_writes: usize,
}

impl Write for DeadPeer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.ok_writes == 0 {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "peer went away"));
        }
        self.ok_writes -= 1;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A runtime that replays a fixed script of generator returns.
pub(super) struct ScriptedRuntime {
    /// Remaining returns, consumed front to back.
    pub(super) returns: Vec<Result<Value, String>>,
    /// How many times [`Runtime::invoke`] was called.
    pub(super) calls: usize,
}

impl ScriptedRuntime {
    /// Build a runtime that yields `frames`, then `nil` forever.
    ///
    /// # Arguments
    ///
    /// * `frames` — Returns to replay, in order.
    ///
    /// # Returns
    ///
    /// The runtime, with `calls` at zero. Infallible.
    pub(super) fn new(frames: Vec<Result<Value, String>>) -> Self {
        ScriptedRuntime {
            returns: frames,
            calls: 0,
        }
    }
}

impl Runtime for ScriptedRuntime {
    fn invoke(&mut self, _callee: &Value, _args: &[Value]) -> Result<Value, String> {
        self.calls += 1;
        if self.returns.is_empty() {
            return Ok(Value::Nil);
        }
        self.returns.remove(0)
    }
}
