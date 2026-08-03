//! How a stream ended, and how many events reached the client.
//!
//! Split from [`super`] so the entry point holds no state definitions. Every
//! variant here except [`StopReason::GeneratorError`] is a *normal* ending: a
//! bounded stream that stops because it hit its bound has behaved correctly, and
//! a client that closes its tab has not caused a server fault.

use std::fmt;

/// Why a stream stopped producing events.
///
/// # Examples
///
/// ```text
/// match outcome.stop {
///     StopReason::Exhausted    => {} // generator returned nil
///     StopReason::Disconnected => {} // client went away; not an error
///     StopReason::MaxEvents    => {} // count bound reached
///     StopReason::MaxDuration  => {} // wall-clock bound reached
///     StopReason::GeneratorError(_) => {} // script raised after the head went out
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StopReason {
    /// The generator returned `nil`, signalling end of stream.
    Exhausted,
    /// A write failed because the peer closed the socket.
    Disconnected,
    /// The configured maximum event count was reached.
    MaxEvents,
    /// The configured maximum wall-clock duration elapsed.
    MaxDuration,
    /// The generator raised. The head is already on the wire, so no status can
    /// be changed; the message is surfaced to the caller instead.
    GeneratorError(String),
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StopReason::Exhausted => write!(f, "generator exhausted"),
            StopReason::Disconnected => write!(f, "client disconnected"),
            StopReason::MaxEvents => write!(f, "max_events reached"),
            StopReason::MaxDuration => write!(f, "max_duration_ms reached"),
            StopReason::GeneratorError(error) => write!(f, "generator error: {error}"),
        }
    }
}

/// Result of serving one streaming response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Outcome {
    /// Number of generator payloads successfully flushed to the client.
    pub(crate) events: u32,
    /// Why the stream stopped.
    pub(crate) stop: StopReason,
}

impl Outcome {
    /// Build an outcome from an event count and a stop reason.
    ///
    /// # Arguments
    ///
    /// * `events` — Payloads flushed.
    /// * `stop` — Why streaming ended.
    ///
    /// # Returns
    ///
    /// The populated [`Outcome`]. Infallible.
    pub(crate) fn new(events: u32, stop: StopReason) -> Self {
        Outcome { events, stop }
    }
}
