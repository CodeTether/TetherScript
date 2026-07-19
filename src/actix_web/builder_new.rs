//! Constructors for static and file-backed route builders.

use std::path::PathBuf;
use std::sync::Arc;

use ::actix_web::http::Method;

use super::builder::ActixPluginBuilder;
use super::source::SourceKind;
use super::state::default_host;

impl ActixPluginBuilder {
    pub(super) fn new(
        path: impl Into<String>,
        method: Method,
        source: impl Into<Arc<str>>,
    ) -> Self {
        Self::base(path, method, SourceKind::Static(source.into()))
    }

    pub(super) fn new_file(
        path: impl Into<String>,
        method: Method,
        source_path: impl Into<PathBuf>,
    ) -> Self {
        Self::base(
            path,
            method,
            SourceKind::File {
                path: source_path.into(),
                hot_reload: true,
            },
        )
    }

    fn base(path: impl Into<String>, method: Method, source: SourceKind) -> Self {
        let path = path.into();
        Self {
            plugin_name: format!("actix:{path}").into(),
            path,
            method,
            source,
            hook: "handle".into(),
            host_factory: default_host(),
        }
    }
}
