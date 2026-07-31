//! Reading PostgreSQL backend messages off a stream.
//!
//! Every backend message is a one-byte tag followed by a self-inclusive 4-byte
//! big-endian length, so the body length is `len - 4`.

use std::io::{Read, Result as IoResult};

/// A single backend message: its tag and raw body bytes.
pub(super) struct Message {
    pub(super) tag: u8,
    pub(super) body: Vec<u8>,
}

/// Read exactly one backend message.
pub(super) fn read<R: Read>(stream: &mut R) -> IoResult<Message> {
    let mut tag = [0u8; 1];
    stream.read_exact(&mut tag)?;
    let mut len = [0u8; 4];
    stream.read_exact(&mut len)?;
    let len = i32::from_be_bytes(len);
    let body_len = (len - 4).max(0) as usize;
    let mut body = vec![0u8; body_len];
    stream.read_exact(&mut body)?;
    Ok(Message { tag: tag[0], body })
}
