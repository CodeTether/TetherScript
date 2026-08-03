//! # HTTP/1.1 chunked transfer encoding
//!
//! A complete, dependency-free codec for the `Transfer-Encoding: chunked` framing of RFC
//! 9112 §7.1, plus the response head a streaming response needs.
//!
//! ## Why this exists
//!
//! `http_serve` writes one complete response and closes the connection, so there is no way
//! to hold a connection open and push bytes as they are produced. Chunked encoding is the
//! transport that makes streaming — Server-Sent Events, log tailing, progressive rendering —
//! possible: each write is self-delimiting, so no total length is needed up front. This
//! module is the codec *only*. It does not touch the server, does not frame SSE events, and
//! adds no builtin.
//!
//! ## Wire format
//!
//! ```text
//! 4\r\nWiki\r\n            one chunk: hex size, CRLF, payload, CRLF
//! 5;ext=1\r\npedia\r\n     extensions after the size are ignored
//! 0\r\n                    the zero chunk ends the body
//! X-Checksum: abc\r\n      optional trailer fields
//! \r\n                     empty line ends the trailer section
//! ```
//!
//! ## Quick start
//!
//! ```
//! use tetherscript::chunked::codec::{decode, encode_chunk, encode_last_chunk, streaming_head};
//!
//! // Writer side: head, then chunks, then the terminator.
//! let head = streaming_head(200, "OK", "text/event-stream", &[]).unwrap();
//! assert!(head.contains("Transfer-Encoding: chunked"));
//! assert!(!head.to_ascii_lowercase().contains("content-length"));
//!
//! let mut wire = Vec::new();
//! wire.extend_from_slice(&encode_chunk(b"Wiki").unwrap());
//! wire.extend_from_slice(&encode_chunk(b"pedia").unwrap());
//! wire.extend_from_slice(&encode_last_chunk(&[]).unwrap());
//! assert_eq!(wire, b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n".to_vec());
//!
//! // Reader side: multi-chunk input concatenates.
//! let body = decode(&wire).unwrap();
//! assert_eq!(body.payload, b"Wikipedia");
//! assert_eq!(body.consumed, wire.len());
//! ```
//!
//! ## Incremental decoding
//!
//! A chunk may arrive split across reads. [`decode`] therefore distinguishes
//! [`ChunkedError::Incomplete`] — a valid prefix, nothing consumed, retry after appending
//! more bytes — from [`ChunkedError::Malformed`], which is terminal. Every proper prefix of
//! a valid body reports `Incomplete`:
//!
//! ```
//! use tetherscript::chunked::codec::{decode, ChunkedError};
//!
//! let wire = b"4\r\nWiki\r\n0\r\n\r\n";
//! for split in 1..wire.len() {
//!     assert!(matches!(decode(&wire[..split]), Err(ChunkedError::Incomplete)), "{split}");
//! }
//! assert_eq!(decode(wire).unwrap().consumed, wire.len());
//! ```
//!
//! ## Security posture
//!
//! Chunked parsing is the classic request-smuggling surface, and every byte here is
//! attacker-controlled. The rules, each documented at its implementation site:
//!
//! * **Strict hex sizes.** A `+`/`-` sign, a `0x` prefix, or any whitespace is rejected
//!   ([`parse_chunk_size`]). These are exactly the shapes two intermediaries read
//!   differently, and disagreement about a size is disagreement about message boundaries.
//! * **No overflow.** Sizes accumulate with `checked_mul`/`checked_add`; an overflowing
//!   size is rejected rather than wrapped to a small number.
//! * **Hard bounds.** 1 MiB per chunk, 8 MiB per body, 256-byte size lines, 1 KiB per
//!   trailer, 4 KiB of trailers, 16 trailers — see [`MAX_CHUNK_BYTES`] and its neighbours.
//!   Exceeding a bound is `Malformed`, never `Incomplete`.
//! * **CRLF only.** A bare LF terminator is rejected ([`crlf_line`]).
//! * **Counted payloads.** A payload is taken by declared length, never by scanning for
//!   CRLF, so payload bytes can never terminate their own chunk early.
//! * **Never both framings.** [`streaming_head`] emits `Transfer-Encoding: chunked` and
//!   never a `Content-Length`, dropping any the caller supplied. A message carrying both is
//!   unrecoverable per RFC 9112 §6.1 and is *the* request-smuggling vector.
//! * **No panics.** Every read goes through `slice::get` or a bounded window, so no index
//!   can be out of range; see the `# Panics` section of each submodule.

mod decode;
mod decoded;
mod encode;
mod encode_body;
mod encode_last;
mod error;
mod extension;
mod head;
mod head_filter;
mod limits;
mod line;
mod payload;
mod size;
mod trailer;
mod trailer_name;
mod trailer_split;

pub use decode::decode;
pub use decoded::DecodedBody;
pub use encode::encode_chunk;
pub use encode_body::encode_body;
pub use encode_last::encode_last_chunk;
pub use error::ChunkedError;
pub use extension::strip_extensions;
pub use head::streaming_head;
pub use head_filter::{check_header, is_reserved_header};
pub use limits::{
    MAX_BODY_BYTES, MAX_CHUNK_BYTES, MAX_SIZE_LINE_BYTES, MAX_TRAILERS, MAX_TRAILER_BYTES,
    MAX_TRAILER_LINE_BYTES,
};
pub use line::crlf_line;
pub use payload::chunk_payload;
pub use size::parse_chunk_size;
pub use trailer::decode_trailers;
pub use trailer_name::check_field as check_trailer_field;
pub use trailer_split::split_field as split_trailer_field;
