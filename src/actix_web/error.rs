//! Actix integration errors and HTTP error conversion.

use std::fmt;

use ::actix_web::{HttpResponse, ResponseError};

/// Failure while loading, invoking, or translating a tetherscript route.
#[derive(Debug)]
pub enum ActixPluginError {
    /// Script loading or hook execution failed.
    Plugin(crate::plugin::PluginError),
    /// The hook returned a value that is not a valid HTTP response map.
    InvalidResponse(String),
    /// Actix could not run the blocking script task.
    Blocking(String),
    /// A file-backed controller could not be read.
    Source(String),
}

impl ActixPluginError {
    pub(super) fn invalid(message: &str) -> Self {
        Self::InvalidResponse(message.into())
    }

    pub(super) fn reject<T>(message: &str) -> Result<T, Self> {
        Err(Self::invalid(message))
    }
}

impl fmt::Display for ActixPluginError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plugin(error) => write!(formatter, "tetherscript plugin: {error}"),
            Self::InvalidResponse(error) => write!(formatter, "tetherscript response: {error}"),
            Self::Blocking(error) => write!(formatter, "tetherscript worker: {error}"),
            Self::Source(error) => write!(formatter, "tetherscript source: {error}"),
        }
    }
}

impl std::error::Error for ActixPluginError {}

impl ResponseError for ActixPluginError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(self.to_string())
    }
}

impl From<crate::plugin::PluginError> for ActixPluginError {
    fn from(error: crate::plugin::PluginError) -> Self {
        Self::Plugin(error)
    }
}
