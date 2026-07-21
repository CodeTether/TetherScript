//! Type-directed resource method dispatch.

use std::time::Instant;

use crate::value::Value;

use super::payload::Payload;

impl Payload {
    pub(super) fn call(
        &mut self,
        name: &str,
        args: &[Value],
        deadline: Option<Instant>,
    ) -> Result<Value, String> {
        match self {
            Self::File(handle) => handle.call(name, args),
            Self::ChildProcess(handle) => handle.call(name, args, deadline),
            Self::TcpStream(handle) => handle.call(name, args),
            Self::TcpListener(handle) => handle.call(name, args),
            Self::RequestBody(handle) => handle.call(name, args),
            Self::ResponseWriter(handle) => handle.call(name, args),
            Self::Task(handle) => handle.call(name, args),
            Self::Timer(handle) => handle.call(name, args),
            Self::Channel(handle) => handle.call(name, args),
            Self::RenderSurface(handle) => handle.call(name, args),
        }
    }
}
