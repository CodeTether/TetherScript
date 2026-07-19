//! Send-safe route configuration shared with Actix workers.

use std::sync::{Arc, Mutex};

use crate::plugin::PluginHost;

use super::script::ScriptState;

pub(crate) type HostFactory = Arc<dyn Fn() -> PluginHost + Send + Sync>;

#[derive(Clone)]
pub(crate) struct RouteState {
    pub cache_key: usize,
    pub plugin_name: Arc<str>,
    pub script: Arc<ScriptState>,
    pub hook: Arc<str>,
    pub host_factory: HostFactory,
    pub reload_lock: Arc<Mutex<()>>,
}

impl RouteState {
    pub fn load(
        &self,
        source: &str,
    ) -> Result<crate::plugin::LoadedPlugin, crate::plugin::PluginError> {
        (self.host_factory)().load_source(self.plugin_name.to_string(), source)
    }
}

pub(crate) fn default_host() -> HostFactory {
    Arc::new(PluginHost::new)
}
