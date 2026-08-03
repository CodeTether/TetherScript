//! WebSocket framing and the RFC 6455 opening handshake.
//!
//! `http_serve` writes one response and closes, so the reference application's six
//! WebSocket files had no path into the port. This is the protocol half: handshake
//! validation, the accept-key transformation, and a frame codec that decodes
//! incrementally from partial socket buffers. Wiring it into the server accept loop is
//! a separate concern and is deliberately not done here.
//!
//! # Layers
//!
//! | Concern | Modules |
//! |---|---|
//! | Opening handshake | `handshake*`, `accept`, `response`, `sha1*` |
//! | Frame header | `header*`, `opcode`, `role` |
//! | Payload | `encode`, `decode`, `mask`, `frame` |
//! | Messages across fragments | `message*` |
//! | Close frames | `close`, `close_code` |
//! | Bounds, validation, errors | `limits`, `validate`, `error*` |
//!
//! Masking is XOR obfuscation, not confidentiality: it exists to defeat proxy cache
//! poisoning. Never mistake a masked frame for an encrypted one.
//!
//! `sha1` here duplicates a private one in `src/rpc_cap.rs`. Consolidating them into a
//! shared module is open follow-up work, recorded in that module's own docs.

#[path = "websocket/accept.rs"]
pub mod accept;
#[path = "websocket/close.rs"]
pub mod close;
#[path = "websocket/close_code.rs"]
pub mod close_code;
#[path = "websocket/decode.rs"]
pub(crate) mod decode;
#[path = "websocket/encode.rs"]
pub mod encode;
#[path = "websocket/error.rs"]
pub mod error;
#[path = "websocket/error_text.rs"]
pub(crate) mod error_text;
#[path = "websocket/error_text_more.rs"]
pub(crate) mod error_text_more;
#[path = "websocket/frame.rs"]
pub mod frame;
#[path = "websocket/handshake.rs"]
pub mod handshake;
#[path = "websocket/handshake_error.rs"]
pub mod handshake_error;
#[path = "websocket/handshake_headers.rs"]
pub(crate) mod handshake_headers;
#[path = "websocket/handshake_key.rs"]
pub mod handshake_key;
#[path = "websocket/header.rs"]
pub mod header;
#[path = "websocket/header_flags.rs"]
pub(crate) mod header_flags;
#[path = "websocket/header_len.rs"]
pub mod header_len;
#[path = "websocket/header_parse.rs"]
pub(crate) mod header_parse;
#[path = "websocket/limits.rs"]
pub mod limits;
#[path = "websocket/mask.rs"]
pub mod mask;
#[path = "websocket/message.rs"]
pub mod message;
#[path = "websocket/message_accept.rs"]
pub(crate) mod message_accept;
#[path = "websocket/message_finish.rs"]
pub(crate) mod message_finish;
#[path = "websocket/opcode.rs"]
pub mod opcode;
#[path = "websocket/response.rs"]
pub mod response;
#[path = "websocket/role.rs"]
pub mod role;
#[path = "websocket/sha1.rs"]
pub mod sha1;
#[path = "websocket/sha1_block.rs"]
pub(crate) mod sha1_block;
#[path = "websocket/validate.rs"]
pub mod validate;
