//! The process-wide store instance and its configuration.
//!
//! A `thread_local!` holder, matching how `src/capability.rs` and the browser JS
//! groups keep interpreter-lifetime state. The runtime is a cooperative
//! single-threaded scheduler, so thread-local *is* process-wide here and no lock
//! is needed; a threaded runtime would replace this file, not the seam.
//!
//! The instance is boxed as `dyn SessionBackend` so swapping in a Redis backend is
//! a one-line change here. The default TTLs mirror the reference application: a
//! 30-minute idle window and a 7-day ceiling.

use std::cell::RefCell;

use super::store_backend::SessionBackend;
use super::store_memory::MemoryBackend;

/// Default idle window: 30 minutes in milliseconds.
pub(super) const DEFAULT_IDLE_MS: i64 = 30 * 60 * 1000;

/// Default absolute ceiling: 7 days in milliseconds, the reference session TTL.
pub(super) const DEFAULT_ABSOLUTE_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// The store: a backend plus the lifetime policy new sessions inherit.
pub(super) struct Store {
    pub(super) backend: Box<dyn SessionBackend>,
    pub(super) idle_ttl_ms: i64,
    pub(super) absolute_ttl_ms: i64,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            backend: Box::new(MemoryBackend::default()),
            idle_ttl_ms: DEFAULT_IDLE_MS,
            absolute_ttl_ms: DEFAULT_ABSOLUTE_MS,
        }
    }
}

thread_local! {
    static STORE: RefCell<Store> = RefCell::new(Store::default());
}

/// Run `body` against the store.
///
/// # Arguments
///
/// * `body` — Closure given exclusive access to the store.
///
/// # Returns
///
/// Whatever `body` returns.
pub(super) fn with<T>(body: impl FnOnce(&mut Store) -> T) -> T {
    STORE.with(|cell| body(&mut cell.borrow_mut()))
}
