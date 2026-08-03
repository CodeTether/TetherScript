//! Buffer bound and the drop decision.
//!
//! ## Why a bound at all
//!
//! An event stream has no `Content-Length` and never ends, so a client that stops
//! reading — a closed laptop lid, a paused tab, a hung proxy — leaves the kernel
//! send buffer full and the server's own buffer growing on every event. With no
//! ceiling that is an unauthenticated memory-exhaustion path: one idle client per
//! megabyte until the process dies.
//!
//! ## The policy
//!
//! The bound is a **byte count of buffered, unwritten frames**, defaulting to
//! [`DEFAULT_BOUND`] (64 KiB). Once the pending bytes reach the bound the stream is
//! *over budget* and the server must **drop the connection**, not block and not
//! keep buffering:
//!
//! * Blocking would let one slow client pin a server thread indefinitely.
//! * Discarding older events silently would hand the client a gap it cannot see.
//!
//! Dropping is honest and recoverable: `EventSource` reconnects on its own and
//! sends `Last-Event-ID`, so a server with a replay log resumes exactly where the
//! client left off. The bound is therefore a *reconnect trigger*, not data loss.
//!
//! Note the comparison is `>=`: reaching the bound is already over budget, so a
//! bound of `0` drops on the first check.

use super::EventStream;

/// Default buffer ceiling: 64 KiB of unwritten frames.
///
/// Chosen to sit above a typical socket send buffer so a briefly-stalled client
/// rides through a burst, and far below anything that matters per-connection:
/// 1,000 stalled clients cost about 64 MiB worst case, which is survivable and
/// observable rather than fatal.
pub const DEFAULT_BOUND: usize = 64 * 1024;

/// Decide whether a stream holding `buffered` bytes must be dropped.
///
/// Free function so the decision can be tested without constructing a stream, and
/// reused by a server that tracks its bytes elsewhere.
///
/// # Arguments
///
/// * `buffered` — Pending unwritten bytes.
/// * `bound` — Ceiling in bytes.
///
/// # Returns
///
/// `true` when `buffered >= bound`, meaning the caller should close the
/// connection and let the client reconnect with `Last-Event-ID`.
///
/// # Examples
///
/// ```rust
/// use tetherscript::sse::backpressure::over_budget;
///
/// assert!(!over_budget(63, 64));
/// assert!(over_budget(64, 64)); // at the bound is already over
/// assert!(over_budget(65, 64));
/// ```
pub fn over_budget(buffered: usize, bound: usize) -> bool {
    buffered >= bound
}

impl EventStream {
    /// Whether this stream is over its own budget.
    ///
    /// # Returns
    ///
    /// `true` when the connection should be dropped; see [`over_budget`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::sse::EventStream;
    ///
    /// let mut stream = EventStream::with_bound(9);
    /// stream.send_data("x"); // exactly 9 bytes: "data: x\n\n"
    /// assert!(stream.should_drop());
    /// ```
    pub fn should_drop(&self) -> bool {
        over_budget(self.buffered(), self.bound())
    }
}
