//! Configuration builder for an Actix plugin route.

use std::sync::Arc;

use ::actix_web::http::Method;

use crate::plugin::PluginHost;

use super::source::SourceKind;
use super::state::HostFactory;

/// Builder for configuring a tetherscript-backed Actix route.
pub struct ActixPluginBuilder {
    pub(super) path: String,
    pub(super) method: Method,
    pub(super) source: SourceKind,
    pub(super) hook: Arc<str>,
    pub(super) plugin_name: Arc<str>,
    pub(super) host_factory: HostFactory,
}

impl ActixPluginBuilder {
    /// Select the script hook invoked for matching requests. Defaults to `handle`.
    pub fn hook(mut self, hook: impl Into<Arc<str>>) -> Self {
        self.hook = hook.into();
        self
    }

    /// Enable or disable automatic reload for a file-backed controller.
    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.source.hot_reload(enabled);
        self
    }

    /// Set the plugin name used in diagnostics.
    pub fn plugin_name(mut self, name: impl Into<Arc<str>>) -> Self {
        self.plugin_name = name.into();
        self
    }

    /// Create a fresh host for each blocking thread, allowing Rust capabilities
    /// backed by database pools or application services to be granted safely.
    pub fn host_factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> PluginHost + Send + Sync + 'static,
    {
        self.host_factory = Arc::new(factory);
        self
    }
}
