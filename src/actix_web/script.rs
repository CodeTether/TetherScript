//! Generation-aware controller source shared by all workers.

use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use super::error::ActixPluginError;
use super::source::SourceKind;

#[derive(Clone)]
pub(super) struct ScriptSnapshot {
    pub generation: u64,
    pub source: Arc<str>,
}

pub(super) struct ScriptState {
    pub source_kind: SourceKind,
    active: RwLock<ScriptSnapshot>,
    modified: RwLock<Option<SystemTime>>,
    reload_error: RwLock<Option<String>>,
}

impl ScriptState {
    pub fn new(source_kind: SourceKind) -> Result<Self, ActixPluginError> {
        let (source, modified) = source_kind.read()?;
        Ok(Self {
            source_kind,
            active: RwLock::new(ScriptSnapshot {
                generation: 1,
                source,
            }),
            modified: RwLock::new(modified),
            reload_error: RwLock::new(None),
        })
    }

    pub fn snapshot(&self) -> ScriptSnapshot {
        self.active.read().expect("script read lock").clone()
    }

    pub fn modified(&self) -> Option<SystemTime> {
        *self.modified.read().expect("modified read lock")
    }

    pub fn activate(&self, source: Arc<str>, modified: Option<SystemTime>) {
        let mut active = self.active.write().expect("script write lock");
        *active = ScriptSnapshot {
            generation: active.generation + 1,
            source,
        };
        *self.modified.write().expect("modified write lock") = modified;
        *self.reload_error.write().expect("reload error lock") = None;
    }

    pub fn reload_error(&self) -> Option<String> {
        self.reload_error.read().expect("reload error lock").clone()
    }

    pub fn reject(&self, error: String, modified: Option<SystemTime>) {
        *self.modified.write().expect("modified write lock") = modified;
        *self.reload_error.write().expect("reload error lock") = Some(error);
    }
}
