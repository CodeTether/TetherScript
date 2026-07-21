//! Resource-kind projection from concrete payloads.

use super::{payload::Payload, ResourceKind};

impl Payload {
    pub(super) fn kind(&self) -> ResourceKind {
        match self {
            Self::File(_) => ResourceKind::File,
            Self::ChildProcess(_) => ResourceKind::ChildProcess,
            Self::TcpStream(_) => ResourceKind::TcpStream,
            Self::TcpListener(_) => ResourceKind::TcpListener,
            Self::RequestBody(_) => ResourceKind::RequestBody,
            Self::ResponseWriter(_) => ResourceKind::ResponseWriter,
            Self::Task(_) => ResourceKind::Task,
            Self::Timer(_) => ResourceKind::Timer,
            Self::Channel(_) => ResourceKind::Channel,
        }
    }
}
