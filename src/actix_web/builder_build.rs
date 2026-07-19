//! Startup validation and construction for plugin routes.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::builder::ActixPluginBuilder;
use super::error::ActixPluginError;
use super::route::ActixPlugin;
use super::script::ScriptState;
use super::state::RouteState;

static NEXT_CACHE_KEY: AtomicUsize = AtomicUsize::new(1);

impl ActixPluginBuilder {
    /// Validate the script and hook, then produce a registrable Actix route.
    pub fn build(self) -> Result<ActixPlugin, ActixPluginError> {
        let script = Arc::new(ScriptState::new(self.source)?);
        let snapshot = script.snapshot();
        super::validate::source(
            &self.host_factory,
            &self.plugin_name,
            &self.hook,
            &snapshot.source,
        )?;
        let state = RouteState {
            cache_key: NEXT_CACHE_KEY.fetch_add(1, Ordering::Relaxed),
            plugin_name: self.plugin_name,
            script,
            hook: self.hook,
            host_factory: self.host_factory,
            reload_lock: Arc::new(Mutex::new(())),
        };
        Ok(ActixPlugin {
            path: self.path,
            method: self.method,
            state,
        })
    }
}
