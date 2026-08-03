//! The successful result of a chunked decode.
//!
//! `consumed` is the load-bearing field for incremental use. A caller holding a buffer that
//! may contain a chunked body *plus* the start of the next pipelined message must know
//! exactly where this body ended; guessing would either drop bytes or replay them as a
//! forged request. On [`ChunkedError::Incomplete`] nothing is consumed at all, so the
//! caller's buffer is safe to append to and retry.
//!
//! [`ChunkedError::Incomplete`]: super::ChunkedError::Incomplete

/// Payload, trailers, and byte count produced by a complete chunked decode.
///
/// # Examples
///
/// ```
/// use tetherscript::chunked::codec::decode;
///
/// let wire = b"5\r\nhello\r\n0\r\nX-Sum: 1\r\n\r\nNEXT";
/// let body = decode(wire).unwrap();
/// assert_eq!(body.payload, b"hello");
/// assert_eq!(body.trailers, vec![("x-sum".to_string(), "1".to_string())]);
/// assert_eq!(&wire[body.consumed..], b"NEXT");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DecodedBody {
    /// Concatenated payloads of every data chunk, in order.
    pub payload: Vec<u8>,
    /// Trailer fields after the zero chunk. Names are lowercased; values are trimmed.
    pub trailers: Vec<(String, String)>,
    /// Bytes of input the body occupied, including the zero chunk and trailer section.
    pub consumed: usize,
}
