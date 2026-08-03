//! Redis client and pool tests. **No Redis server, and no RESP codec.**
//!
//! Everything here runs against two doubles that stand in for the client's two
//! injected boundaries:
//!
//! * `FakeCodec` implements `RespCodec`. It encodes real RESP command arrays, so
//!   the request assertions below are byte-exact and will still be byte-exact once
//!   the real `src/resp/codec.rs` is wired in behind the same trait. Its decoder
//!   is a deliberately small RESP2 reader, and it reports a truncated buffer as
//!   `Ok(None)` — the `Incomplete` contract the boundary requires.
//! * `ScriptedTransport` implements `Transport`. It records every byte written and
//!   replays a scripted sequence of reads, including a mid-stream I/O failure and
//!   a mid-reply EOF, which is how the discard rule is tested without a server.
//!
//! This file is a single integration test binary because the two doubles are only
//! meaningful together with the cases that use them; `check_file_limits.sh` scopes
//! its 50-line rule to `src/**/*.rs`, and the existing `tests/*.rs` in this
//! repository are of comparable size.
//!
//! ## One wiring prerequisite
//!
//! These tests need `src/redis.rs` to declare the client module, since this crate
//! delivers `src/redis/*.rs` without the parent file:
//!
//! ```rust,ignore
//! #[path = "redis/client.rs"]
//! pub mod client;
//! ```
//!
//! Nothing else is required: no server, no codec, no feature flag.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use tetherscript::redis::client::{
    ClientError, Config, Connection, Pool, Reply, RespCodec, Transport, Ttl,
};

// ----------------------------------------------------------------- the doubles

/// One scripted read outcome.
#[derive(Clone)]
enum Step {
    /// Deliver these bytes.
    Bytes(Vec<u8>),
    /// Fail the read, as a timeout or reset would.
    Fail(&'static str),
    /// Report end of stream.
    Eof,
}

/// Deliver a whole reply in one read.
fn reply(bytes: &str) -> Step {
    Step::Bytes(bytes.as_bytes().to_vec())
}

/// A `Transport` that records writes and replays scripted reads.
struct ScriptedTransport {
    written: Rc<RefCell<Vec<u8>>>,
    reads: VecDeque<Step>,
    write_fails: bool,
}

impl Transport for ScriptedTransport {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), ClientError> {
        if self.write_fails {
            return Err(ClientError::Transport("write command: broken pipe".into()));
        }
        self.written.borrow_mut().extend_from_slice(bytes);
        Ok(())
    }

    fn read(&mut self, into: &mut [u8]) -> Result<usize, ClientError> {
        match self.reads.pop_front() {
            None | Some(Step::Eof) => Ok(0),
            Some(Step::Fail(detail)) => {
                Err(ClientError::Transport(format!("read reply: {detail}")))
            }
            Some(Step::Bytes(bytes)) => {
                let take = bytes.len().min(into.len());
                into[..take].copy_from_slice(&bytes[..take]);
                Ok(take)
            }
        }
    }
}

/// A `RespCodec` with a real encoder and a minimal RESP2 decoder.
struct FakeCodec;

impl RespCodec for FakeCodec {
    fn encode_command(&self, args: &[&[u8]]) -> Result<Vec<u8>, ClientError> {
        if args.is_empty() {
            return Err(ClientError::Protocol("encode: no command name".into()));
        }
        let mut out = format!("*{}\r\n", args.len()).into_bytes();
        for arg in args {
            out.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
            out.extend_from_slice(arg);
            out.extend_from_slice(b"\r\n");
        }
        Ok(out)
    }

    fn decode_reply(&self, buf: &[u8]) -> Result<Option<(Reply, usize)>, ClientError> {
        parse(buf, 0)
    }
}

/// Parse one reply starting at `at`, returning the offset just past it.
fn parse(buf: &[u8], at: usize) -> Result<Option<(Reply, usize)>, ClientError> {
    let Some(end) = crlf(buf, at) else {
        return Ok(None);
    };
    let line = String::from_utf8_lossy(&buf[at + 1..end]).into_owned();
    let next = end + 2;
    match buf[at] {
        b'+' => Ok(Some((Reply::Status(line), next))),
        b'-' => {
            let (kind, message) = line.split_once(' ').unwrap_or((line.as_str(), ""));
            let error = Reply::Error {
                kind: kind.to_string(),
                message: message.to_string(),
            };
            Ok(Some((error, next)))
        }
        b':' => Ok(Some((Reply::Integer(number(&line)?), next))),
        b'$' => bulk(buf, number(&line)?, next),
        b'*' => array(buf, number(&line)?, next),
        other => Err(ClientError::Protocol(format!(
            "unknown reply type byte {other:?}"
        ))),
    }
}

/// Offset of the CRLF terminating the line beginning at `at`, if present.
fn crlf(buf: &[u8], at: usize) -> Option<usize> {
    if at >= buf.len() {
        return None;
    }
    (at + 1..buf.len().saturating_sub(1)).find(|&i| &buf[i..i + 2] == b"\r\n")
}

/// Parse a header count or an integer payload.
fn number(line: &str) -> Result<i64, ClientError> {
    line.parse()
        .map_err(|_| ClientError::Protocol(format!("`{line}` is not an integer")))
}

/// Read a `$`-prefixed body of declared length `len`, starting at `from`.
fn bulk(buf: &[u8], len: i64, from: usize) -> Result<Option<(Reply, usize)>, ClientError> {
    if len < 0 {
        return Ok(Some((Reply::Nil, from)));
    }
    let len = len as usize;
    if buf.len() < from + len + 2 {
        return Ok(None);
    }
    let body = buf[from..from + len].to_vec();
    Ok(Some((Reply::Bulk(body), from + len + 2)))
}

/// Read `count` array elements starting at `from`.
fn array(buf: &[u8], count: i64, from: usize) -> Result<Option<(Reply, usize)>, ClientError> {
    if count < 0 {
        return Ok(Some((Reply::Nil, from)));
    }
    let mut items = Vec::new();
    let mut at = from;
    for _ in 0..count {
        match parse(buf, at)? {
            Some((item, next)) => {
                items.push(item);
                at = next;
            }
            None => return Ok(None),
        }
    }
    Ok(Some((Reply::Array(items), at)))
}

// ------------------------------------------------------------------- fixtures

/// A connection over a scripted transport, plus the log of what it wrote.
fn connection(steps: Vec<Step>) -> (Connection, Rc<RefCell<Vec<u8>>>) {
    connection_with(steps, false)
}

/// As `connection`, optionally making every write fail.
fn connection_with(steps: Vec<Step>, write_fails: bool) -> (Connection, Rc<RefCell<Vec<u8>>>) {
    let written = Rc::new(RefCell::new(Vec::new()));
    let transport = ScriptedTransport {
        written: Rc::clone(&written),
        reads: steps.into(),
        write_fails,
    };
    (
        Connection::from_parts(Box::new(transport), Box::new(FakeCodec)),
        written,
    )
}

/// The bytes a connection wrote, as a string for readable assertions.
fn wire(written: &Rc<RefCell<Vec<u8>>>) -> String {
    String::from_utf8(written.borrow().clone()).expect("requests are ASCII here")
}

/// A pool of `max` connections whose transports each replay `steps`.
///
/// The returned counter records how many connections were actually opened, which
/// is how reuse is distinguished from silent reconnection.
fn pool(max: usize, steps: Vec<Step>) -> (Pool, Rc<RefCell<usize>>) {
    let opened = Rc::new(RefCell::new(0));
    let counter = Rc::clone(&opened);
    let connector = move |_: &Config| -> Result<Connection, ClientError> {
        *counter.borrow_mut() += 1;
        Ok(connection(steps.clone()).0)
    };
    (
        Pool::new(Config::default(), max, Box::new(connector)),
        opened,
    )
}

// ------------------------------------------------------- encoded request shape

#[test]
fn get_sends_a_two_element_command_array() {
    let (mut connection, written) = connection(vec![reply("$5\r\nhello\r\n")]);
    assert_eq!(connection.get(b"session:42").unwrap(), Some(b"hello".to_vec()));
    assert_eq!(wire(&written), "*2\r\n$3\r\nGET\r\n$10\r\nsession:42\r\n");
}

#[test]
fn set_sends_key_and_value() {
    let (mut connection, written) = connection(vec![reply("+OK\r\n")]);
    connection.set(b"k", b"v").unwrap();
    assert_eq!(wire(&written), "*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n");
}

#[test]
fn set_ex_sends_the_expiry_atomically_in_one_command() {
    let (mut connection, written) = connection(vec![reply("+OK\r\n")]);
    connection.set_ex(b"k", b"v", 60).unwrap();
    assert_eq!(
        wire(&written),
        "*5\r\n$3\r\nSET\r\n$1\r\nk\r\n$1\r\nv\r\n$2\r\nEX\r\n$2\r\n60\r\n"
    );
}

#[test]
fn del_is_variadic_and_returns_the_count_removed() {
    let (mut connection, written) = connection(vec![reply(":1\r\n")]);
    assert_eq!(connection.del(&[&b"a"[..], &b"b"[..]]).unwrap(), 1);
    assert_eq!(wire(&written), "*3\r\n$3\r\nDEL\r\n$1\r\na\r\n$1\r\nb\r\n");
}

#[test]
fn exists_sends_every_key() {
    let (mut connection, written) = connection(vec![reply(":0\r\n")]);
    assert_eq!(connection.exists(&[&b"a"[..]]).unwrap(), 0);
    assert_eq!(wire(&written), "*2\r\n$6\r\nEXISTS\r\n$1\r\na\r\n");
}

#[test]
fn a_keyless_del_fails_locally_without_a_round_trip() {
    let (mut connection, written) = connection(vec![]);
    let error = connection.del(&[]).unwrap_err();
    assert_eq!(
        error,
        ClientError::Protocol("DEL: at least one key is required".into())
    );
    assert_eq!(wire(&written), "");
}

#[test]
fn expire_reports_whether_the_key_existed() {
    let (mut connection, written) = connection(vec![reply(":1\r\n"), reply(":0\r\n")]);
    assert!(connection.expire(b"k", 30).unwrap());
    assert_eq!(
        wire(&written),
        "*3\r\n$6\r\nEXPIRE\r\n$1\r\nk\r\n$2\r\n30\r\n"
    );
    assert!(!connection.expire(b"k", 30).unwrap());
}

#[test]
fn incr_and_incr_by_send_their_documented_forms() {
    let (mut connection, written) = connection(vec![reply(":1\r\n"), reply(":6\r\n")]);
    assert_eq!(connection.incr(b"rate:ip").unwrap(), 1);
    assert_eq!(connection.incr_by(b"rate:ip", 5).unwrap(), 6);
    assert_eq!(
        wire(&written),
        concat!(
            "*2\r\n$4\r\nINCR\r\n$7\r\nrate:ip\r\n",
            "*3\r\n$6\r\nINCRBY\r\n$7\r\nrate:ip\r\n$1\r\n5\r\n"
        )
    );
}

#[test]
fn the_generic_command_escape_hatch_sends_arbitrary_commands() {
    let (mut connection, written) = connection(vec![reply("+PONG\r\n")]);
    let pong = connection.command(&[&b"PING"[..]]).unwrap();
    assert_eq!(pong, Reply::Status("PONG".into()));
    assert_eq!(wire(&written), "*1\r\n$4\r\nPING\r\n");
}

#[test]
fn binary_keys_and_values_survive_encoding() {
    let (mut connection, written) = connection(vec![reply("+OK\r\n")]);
    connection.set(b"k", b"a\r\nb").unwrap();
    assert!(written.borrow().ends_with(b"$4\r\na\r\nb\r\n"));
}

// --------------------------------------------- a missing key is not empty data

#[test]
fn a_missing_key_is_none_and_an_empty_value_is_some_empty() {
    let (mut connection, _) = connection(vec![reply("$-1\r\n"), reply("$0\r\n\r\n")]);
    assert_eq!(connection.get(b"absent").unwrap(), None);
    assert_eq!(connection.get(b"empty").unwrap(), Some(Vec::new()));
}

#[test]
fn the_reply_model_keeps_nil_and_empty_bulk_distinct() {
    assert_ne!(Reply::Nil, Reply::Bulk(Vec::new()));
    assert!(Reply::Nil.is_nil());
    assert!(!Reply::Bulk(Vec::new()).is_nil());
}

// ------------------------------------------------------------- ttl sentinels

#[test]
fn ttl_maps_the_negative_sentinels_instead_of_returning_them() {
    let (mut connection, written) = connection(vec![
        reply(":42\r\n"),
        reply(":-1\r\n"),
        reply(":-2\r\n"),
    ]);
    assert_eq!(connection.ttl(b"k").unwrap(), Ttl::Seconds(42));
    assert_eq!(connection.ttl(b"k").unwrap(), Ttl::Persistent);
    assert_eq!(connection.ttl(b"k").unwrap(), Ttl::Missing);
    assert!(wire(&written).starts_with("*2\r\n$3\r\nTTL\r\n$1\r\nk\r\n"));
}

#[test]
fn ttl_persistent_and_missing_are_not_interchangeable() {
    assert_ne!(Ttl::Persistent, Ttl::Missing);
    assert_eq!(Ttl::from_reply(0), Ttl::Seconds(0));
}

// --------------------------------------------------- errors: data versus fault

#[test]
fn an_error_reply_surfaces_as_an_error_but_keeps_the_connection() {
    let (mut connection, _) = connection(vec![
        reply("-WRONGTYPE Operation against a key holding the wrong kind of value\r\n"),
        reply("$2\r\nok\r\n"),
    ]);
    let error = connection.get(b"a-list").unwrap_err();
    assert!(error.is_server_error());
    assert_eq!(error.kind_for_test(), "WRONGTYPE");
    // The exchange completed, so the stream is still aligned: the next command
    // reads its own reply, not this one's leftovers.
    assert!(!error.discards_connection());
    assert_eq!(connection.get(b"k").unwrap(), Some(b"ok".to_vec()));
}

#[test]
fn a_wrong_reply_type_is_a_type_error_not_a_transport_fault() {
    let (mut connection, _) = connection(vec![reply(":7\r\n")]);
    let error = connection.get(b"k").unwrap_err();
    assert!(matches!(error, ClientError::UnexpectedType(_)));
    assert!(!error.discards_connection());
}

#[test]
fn a_read_failure_is_a_transport_error_that_discards() {
    let (mut connection, _) = connection(vec![Step::Fail("timed out")]);
    let error = connection.get(b"k").unwrap_err();
    assert_eq!(
        error,
        ClientError::Transport("read reply: timed out".into())
    );
    assert!(error.discards_connection());
}

#[test]
fn a_write_failure_is_a_transport_error_that_discards() {
    let (mut connection, _) = connection_with(vec![], true);
    let error = connection.get(b"k").unwrap_err();
    assert!(error.discards_connection());
}

#[test]
fn an_eof_mid_reply_is_a_transport_error_naming_the_outstanding_reply() {
    let (mut connection, _) = connection(vec![reply("$5\r\nhel"), Step::Eof]);
    let error = connection.get(b"k").unwrap_err();
    assert_eq!(
        error,
        ClientError::Transport(
            "server closed the connection with a reply outstanding".into()
        )
    );
}

#[test]
fn a_reply_split_across_reads_is_reassembled() {
    let (mut connection, _) = connection(vec![reply("$5\r\nhel"), reply("lo\r\n")]);
    assert_eq!(connection.get(b"k").unwrap(), Some(b"hello".to_vec()));
}

#[test]
fn a_pipelined_remainder_is_left_for_the_next_command() {
    // Both replies arrive in one read; the second must not be lost or mixed in.
    let (mut connection, _) = connection(vec![reply("$1\r\na\r\n$1\r\nb\r\n")]);
    assert_eq!(connection.get(b"k1").unwrap(), Some(b"a".to_vec()));
    assert_eq!(connection.get(b"k2").unwrap(), Some(b"b".to_vec()));
}

#[test]
fn undecodable_bytes_are_a_protocol_error_that_discards() {
    let (mut connection, _) = connection(vec![reply("?nonsense\r\n")]);
    let error = connection.get(b"k").unwrap_err();
    assert!(matches!(error, ClientError::Protocol(_)));
    assert!(error.discards_connection());
}

// ------------------------------------------------------------------- the pool

#[test]
fn a_new_pool_opens_nothing() {
    let (pool, opened) = pool(4, vec![]);
    assert_eq!(pool.size(), 0);
    assert_eq!(*opened.borrow(), 0);
}

#[test]
fn a_released_connection_is_reused_rather_than_reopened() {
    let (pool, opened) = pool(4, vec![reply("+OK\r\n"), reply("+OK\r\n")]);
    pool.with_connection(|c| c.set(b"k", b"v")).unwrap();
    pool.with_connection(|c| c.set(b"k", b"v")).unwrap();
    assert_eq!(*opened.borrow(), 1);
    assert_eq!(pool.size(), 1);
}

#[test]
fn acquire_then_release_hands_the_same_slot_back() {
    let (pool, opened) = pool(4, vec![reply("+OK\r\n")]);
    let leased = pool.acquire().unwrap();
    assert_eq!(pool.size(), 1);
    pool.release(leased);
    let again = pool.acquire().unwrap();
    assert_eq!(*opened.borrow(), 1);
    pool.release(again);
}

#[test]
fn exhaustion_names_the_limit_because_the_fix_is_a_bigger_pool() {
    let (pool, _) = pool(1, vec![reply("+OK\r\n")]);
    let held = pool.acquire().unwrap();
    let error = pool.acquire().unwrap_err();
    assert_eq!(error, ClientError::PoolExhausted { in_use: 1, max: 1 });
    assert!(error.to_string().contains("max 1"));
    assert!(!error.discards_connection());
    pool.release(held);
}

#[test]
fn a_zero_sized_pool_is_clamped_to_one_usable_connection() {
    let (pool, _) = pool(0, vec![reply("+OK\r\n")]);
    pool.with_connection(|c| c.set(b"k", b"v")).unwrap();
    assert_eq!(pool.size(), 1);
}

#[test]
fn a_server_error_reply_keeps_the_connection_in_the_pool() {
    let (pool, opened) = pool(4, vec![reply("-ERR nope\r\n"), reply("+OK\r\n")]);
    let error = pool.with_connection(|c| c.get(b"k")).unwrap_err();
    assert!(error.is_server_error());
    assert_eq!(pool.size(), 1);
    // Reused, not reopened: the exchange completed, so the stream is aligned.
    pool.with_connection(|c| c.set(b"k", b"v")).unwrap();
    assert_eq!(*opened.borrow(), 1);
}

#[test]
fn a_transport_failure_discards_the_connection_from_the_pool() {
    let (pool, opened) = pool(4, vec![Step::Fail("connection reset")]);
    let error = pool.with_connection(|c| c.get(b"k")).unwrap_err();
    assert!(error.discards_connection());
    assert_eq!(pool.size(), 0);
    // The slot was freed, so the next lease opens a clean replacement.
    let _ = pool.with_connection(|c| c.get(b"k"));
    assert_eq!(*opened.borrow(), 2);
}

#[test]
fn discard_frees_a_slot_without_underflowing() {
    let (pool, _) = pool(2, vec![]);
    pool.discard();
    assert_eq!(pool.size(), 0);
}

// ------------------------------------------------- the password stays a secret

#[test]
fn the_password_never_appears_in_config_debug_output() {
    let config = Config {
        password: Some("hunter2-correct-horse".into()),
        ..Config::default()
    };
    let rendered = format!("{config:?}");
    assert!(!rendered.contains("hunter2-correct-horse"));
    assert!(rendered.contains("<redacted>"));
    assert!(rendered.contains("127.0.0.1"));
}

#[test]
fn an_absent_password_renders_as_none_not_as_a_redaction() {
    let rendered = format!("{:?}", Config::default());
    assert!(rendered.contains("password: None"));
}

#[test]
fn connection_and_pool_debug_output_reveal_nothing() {
    let (connection, _) = connection(vec![]);
    assert_eq!(format!("{connection:?}"), "Connection(..)");
    let (pool, _) = pool(3, vec![]);
    let rendered = format!("{pool:?}");
    assert!(rendered.contains("max 3"));
    assert!(!rendered.contains("password"));
}

// ------------------------------------------------------------------- helpers

/// Test-local accessor for a server error's kind, so the assertion above reads
/// directly instead of matching a struct variant inline.
trait ServerErrorKind {
    fn kind_for_test(&self) -> &str;
}

impl ServerErrorKind for ClientError {
    fn kind_for_test(&self) -> &str {
        match self {
            ClientError::Server { kind, .. } => kind,
            other => panic!("expected a server error, got {other:?}"),
        }
    }
}
