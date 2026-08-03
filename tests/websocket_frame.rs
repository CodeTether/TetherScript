//! Conformance tests for the WebSocket frame codec and opening handshake.
//!
//! Coverage is organized by the layer under test, one file per concern, matching
//! the repository's `#[path]` test-splitting convention (see
//! `tests/browser_agent_e2e.rs`) and keeping every file inside the 50-line limit.
//!
//! | Module | What it locks in |
//! |---|---|
//! | `wire` | Shared frame-building helpers, written byte-by-byte |
//! | `handshake_tests` | RFC 6455 §1.3 key/accept vector, header validation |
//! | `length_tests` | All three payload-length forms and encoder minimality |
//! | `mask_tests` | Masking round-trip and the masking-direction rules |
//! | `fragment_tests` | Reassembly across fragments |
//! | `sequence_tests` | Fragmented control frames, illegal frame ordering |
//! | `reject_tests` | Reserved opcodes and non-zero RSV bits |
//! | `bound_tests` | 64-bit MSB, payload cap, control cap, minimal lengths |
//! | `message_bound_tests` | The reassembled-message cap |
//! | `utf8_close_tests` | Text UTF-8 validity at frame and message level |
//! | `close_body_tests` | Close code/reason parsing and round-trip |
//! | `close_code_tests` | The close-code allow list, incl. 1005/1006/1015 |
//! | `incomplete_tests` | Every prefix of a valid frame is `Incomplete` |
//!
//! **Note for the integrator:** these tests import `tetherscript::websocket`, so
//! they require the module declaration (`pub mod websocket;` in `src/lib.rs` plus
//! a `src/websocket.rs` declaring the submodules). That wiring is owned by the
//! server agent, per the task split, so this file is inert until it lands.

#[path = "websocket_frame/wire.rs"]
mod wire;

#[path = "websocket_frame/bound_tests.rs"]
mod bound_tests;
#[path = "websocket_frame/close_body_tests.rs"]
mod close_body_tests;
#[path = "websocket_frame/close_code_tests.rs"]
mod close_code_tests;
#[path = "websocket_frame/fragment_tests.rs"]
mod fragment_tests;
#[path = "websocket_frame/handshake_tests.rs"]
mod handshake_tests;
#[path = "websocket_frame/incomplete_tests.rs"]
mod incomplete_tests;
#[path = "websocket_frame/length_tests.rs"]
mod length_tests;
#[path = "websocket_frame/mask_tests.rs"]
mod mask_tests;
#[path = "websocket_frame/message_bound_tests.rs"]
mod message_bound_tests;
#[path = "websocket_frame/reject_tests.rs"]
mod reject_tests;
#[path = "websocket_frame/sequence_tests.rs"]
mod sequence_tests;
#[path = "websocket_frame/utf8_close_tests.rs"]
mod utf8_close_tests;
