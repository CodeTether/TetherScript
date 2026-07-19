//! Static and file-backed controller source definitions.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use super::error::ActixPluginError;

pub(super) enum SourceKind {
    Static(Arc<str>),
    File { path: PathBuf, hot_reload: bool },
}

impl SourceKind {
    pub fn read(&self) -> Result<(Arc<str>, Option<SystemTime>), ActixPluginError> {
        match self {
            Self::Static(source) => Ok((source.clone(), None)),
            Self::File { path, .. } => {
                let source = std::fs::read_to_string(path).map_err(|error| {
                    ActixPluginError::Source(format!("{}: {error}", path.display()))
                })?;
                let modified = std::fs::metadata(path)
                    .and_then(|metadata| metadata.modified())
                    .map_err(|error| {
                        ActixPluginError::Source(format!("{}: {error}", path.display()))
                    })?;
                Ok((source.into(), Some(modified)))
            }
        }
    }

    pub fn hot_reload(&mut self, enabled: bool) {
        if let Self::File { hot_reload, .. } = self {
            *hot_reload = enabled;
        }
    }

    pub fn reloads(&self) -> bool {
        matches!(
            self,
            Self::File {
                hot_reload: true,
                ..
            }
        )
    }
}
