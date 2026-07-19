//! Validated route registration with Actix Web.

use std::path::Path;
use std::sync::Arc;

use ::actix_web::{http::Method, web};

use super::builder::ActixPluginBuilder;
use super::state::RouteState;

/// A validated Actix route backed by a tetherscript hook.
#[derive(Clone)]
pub struct ActixPlugin {
    pub(super) path: String,
    pub(super) method: Method,
    pub(super) state: RouteState,
}

impl ActixPlugin {
    /// Start building a route for `method` and `path` from script `source`.
    pub fn builder(
        path: impl Into<String>,
        method: Method,
        source: impl Into<Arc<str>>,
    ) -> ActixPluginBuilder {
        ActixPluginBuilder::new(path, method, source)
    }

    /// Build a hot-reloading route from a tetherscript source file.
    pub fn from_file(
        path: impl Into<String>,
        method: Method,
        source_path: impl AsRef<Path>,
    ) -> ActixPluginBuilder {
        ActixPluginBuilder::new_file(path, method, source_path.as_ref().to_path_buf())
    }

    /// Return the latest rejected hot-reload diagnostic, if any.
    pub fn reload_error(&self) -> Option<String> {
        self.state.script.reload_error()
    }

    /// Register this route on an Actix [`web::ServiceConfig`].
    pub fn configure(&self, config: &mut web::ServiceConfig) {
        let state = web::Data::new(self.state.clone());
        let route = web::method(self.method.clone()).to(super::execute::handle);
        config.service(web::resource(&self.path).app_data(state).route(route));
    }
}
