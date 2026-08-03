//! Dependency-free cooperative async scheduling primitives.
//!
//! These types track task readiness, completed results, and join waiters
//! without pulling in Tokio or any external event loop.

mod finish;
// Bounded channels, the one stdlib gap AGENTS.md names alongside the async scheduler:
// `spawn`/`join`/`select` existed with no way for tasks to pass values.
pub mod channel;
mod join;
mod queue;
pub(crate) mod runtime;
mod spawn;
mod task;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use queue::Scheduler;
#[allow(unused_imports)]
pub use task::{Task, TaskState};
