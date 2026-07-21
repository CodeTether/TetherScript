//! Host cancellation for cancellable payloads.

use super::payload::Payload;

impl Payload {
    pub(super) fn cancel(&mut self) -> Result<(), String> {
        match self {
            Self::ChildProcess(handle) => handle.cancel(),
            Self::TcpStream(handle) => handle.cancel(),
            _ => Ok(()),
        }
    }
}
