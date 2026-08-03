//! Bounded channels for the cooperative scheduler.
//!
//! `spawn`, `join`, and `select` let tasks *start* and *finish* together, but
//! give them no way to stream values while they run, so every producer/consumer
//! program has to be written as a batch. This module closes that gap with a
//! bounded first-in first-out channel: one [`Sender`] half, one [`Receiver`]
//! half, and a fixed-size buffer between them.
//!
//! ## Why bounded, never unbounded
//!
//! Boundedness is the feature, not a limitation. An unbounded channel silently
//! converts a fast producer into unbounded memory growth: the producer never
//! feels the consumer's slowness, so the queue is the only thing absorbing the
//! mismatch until the process dies. A bounded channel makes the mismatch visible
//! exactly where it happens — [`Sender::send`] reports [`SendOutcome::Parked`]
//! once the buffer is full, so the producer stops producing until the consumer
//! catches up. That is backpressure, and it is why this channel exists.
//!
//! ## How parking works here
//!
//! The scheduler in this crate is cooperative and single-threaded: a task runs
//! until it returns or reaches an async point. This module therefore never
//! blocks. It *parks*: the task id goes into the channel's waiter queue and into
//! a park table, and the operation returns `Parked`. When the counterpart
//! operation makes progress it moves waiter ids onto a wakeup queue, which the
//! scheduler drains with [`take_wakeups`] and feeds into its ready queue through
//! the existing `try_wake` path. Parking and waking are expressed in task ids —
//! the same currency the existing scheduler already uses.
//!
//! ## Deadlock
//!
//! Every wake source here is in-process and enumerable, so an all-parked state
//! is *provably* terminal rather than merely suspicious. See [`detect_deadlock`].
//!
//! ## Layout
//!
//! `internals` holds the channel mechanism; `builtins` holds the script-facing
//! `chan_*` functions that wrap it. Nothing else in the crate is touched.
//!
//! # Examples
//!
//! ```
//! use tetherscript::scheduler::channel::{self, RecvOutcome, SendOutcome};
//! use tetherscript::value::Value;
//!
//! let (tx, rx) = channel::bounded(1, "demo")?;
//! assert_eq!(tx.send(&Value::Int(1), 1)?, SendOutcome::Sent);
//! // The single slot is taken, so the next send parks instead of growing.
//! assert_eq!(tx.send(&Value::Int(2), 1)?, SendOutcome::Parked);
//! assert!(matches!(rx.recv(2), RecvOutcome::Value(Value::Int(1))));
//! // Receiving freed a slot, so the parked sender was queued for wakeup.
//! assert_eq!(channel::take_wakeups(), vec![1]);
//! # Ok::<(), String>(())
//! ```

#[path = "channel_builtins.rs"]
mod builtins;
#[path = "channel_internals.rs"]
mod internals;

#[cfg(test)]
#[path = "channel_tests.rs"]
mod tests;

pub use builtins::builtin::{channel_open, channel_send};
pub use builtins::builtin_close::channel_close;
pub use builtins::builtin_deadlock::channel_deadlock;
pub use builtins::builtin_drop::channel_drop_receiver;
pub use builtins::builtin_ops::channel_recv;
pub use builtins::builtin_query::{channel_ended, channel_len};
pub use builtins::builtin_select::channel_select;
pub use internals::bounded::bounded;
pub use internals::cancel::cancel_task;
pub use internals::current::{current_task, set_current_task};
pub use internals::deadlock::detect_deadlock;
pub use internals::endpoint::{Receiver, Sender};
pub use internals::parked::parked_tasks;
pub use internals::recv::RecvOutcome;
pub use internals::select::{select_recv, SelectOutcome};
pub use internals::send::SendOutcome;
pub use internals::wake::take_wakeups;
