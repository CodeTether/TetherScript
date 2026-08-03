//! Process-local in-memory implementation of [`SessionBackend`].
//!
//! # Documented limitations
//!
//! * **Process-local.** The map lives in the interpreter's own memory.
//! * **Lost on restart.** No persistence; every session vanishes with the process,
//!   so a deploy logs every user out.
//! * **Not shared across processes.** Two `tetherscript` processes, or two workers
//!   behind a load balancer, see entirely separate stores, so a user routed to the
//!   other worker appears logged out.
//! * **Grows until swept.** Expired records occupy memory until `store_sweep`
//!   runs; there is no background reaper, because the runtime is a cooperative
//!   single-threaded scheduler with no timer thread to host one.
//!
//! It exists so the abstraction is usable and testable *today*, and so the seam
//! has a real implementation proving it is not shaped around one backend. A Redis
//! backend replaces this file and nothing else in the group.

use std::collections::HashMap;

use super::store_backend::SessionBackend;
use super::store_expiry::evaluate;
use super::store_record::Record;

/// A `HashMap`-backed store.
///
/// Iteration order is unspecified, which is fine: nothing here depends on order,
/// and both sweeps scan rather than seek.
#[derive(Default)]
pub(super) struct MemoryBackend {
    records: HashMap<String, Record>,
}

impl SessionBackend for MemoryBackend {
    fn create(&mut self, record: Record) -> Result<(), String> {
        if self.records.contains_key(&record.id) {
            // Unreachable with a working CSPRNG. Loud rather than silent: the
            // quiet alternative hands one user another user's session.
            return Err("store_create: generated session id already exists".into());
        }
        self.records.insert(record.id.clone(), record);
        Ok(())
    }

    fn load(&self, id: &str) -> Result<Option<Record>, String> {
        Ok(self.records.get(id).cloned())
    }

    fn save(&mut self, record: Record) -> Result<(), String> {
        if !self.records.contains_key(&record.id) {
            return Err("store_save: no session with that id".into());
        }
        self.records.insert(record.id.clone(), record);
        Ok(())
    }

    fn delete(&mut self, id: &str) -> Result<bool, String> {
        Ok(self.records.remove(id).is_some())
    }

    fn delete_subject(&mut self, subject: &str) -> Result<usize, String> {
        let before = self.records.len();
        self.records.retain(|_, held| held.subject != subject);
        Ok(before - self.records.len())
    }

    fn count(&self) -> Result<usize, String> {
        Ok(self.records.len())
    }

    fn sweep(&mut self, now_ms: i64) -> Result<usize, String> {
        let before = self.records.len();
        self.records
            .retain(|_, held| evaluate(held, now_ms).is_none());
        Ok(before - self.records.len())
    }
}
