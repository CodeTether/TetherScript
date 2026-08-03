//! Live coverage for the pooled Redis handler.
//!
//! These tests need a real server: pooling correctness is about what happens to a
//! socket after a failure, which a mock cannot prove. They skip cleanly unless
//! `REDIS_TEST_URL` is set, so the default `cargo test` run stays hermetic.
//!
//! ```text
//! docker run -d --rm --name ts_redis_test -p 56379:6379 redis:7
//! REDIS_TEST_URL=127.0.0.1:56379 cargo test --test redis_pool_live
//! ```

use tetherscript::redis::{Config, RedisHandler};
use tetherscript::value::Value;

/// Build a handler, or `None` when live testing is not enabled.
fn handler() -> Option<RedisHandler> {
    let target = std::env::var("REDIS_TEST_URL").ok()?;
    let target = target.trim_start_matches("redis://").trim_end_matches('/');
    let (host, port) = target.rsplit_once(':')?;
    let config = Config {
        host: host.to_string(),
        port: port.parse().ok()?,
        username: None,
        password: None,
        database: 0,
        tls: false,
    };
    Some(RedisHandler::connect(&config).expect("connect to REDIS_TEST_URL should succeed"))
}

/// A key unique to this test binary run and to `label`.
fn key(label: &str) -> String {
    format!("tetherscript:test:{}:{}", std::process::id(), label)
}

fn text(value: &Value) -> String {
    match value {
        Value::Str(s) => s.to_string(),
        other => panic!("expected a string, got {}", other.type_name()),
    }
}

fn int(value: &Value) -> i64 {
    match value {
        Value::Int(n) => *n,
        other => panic!("expected an int, got {}", other.type_name()),
    }
}

/// A written value must read back unchanged.
#[test]
fn set_then_get_round_trips() {
    let Some(handler) = handler() else { return };
    let k = key("roundtrip");
    handler.set(&k, b"hello", None).expect("SET should succeed");
    assert_eq!(text(&handler.get(&k).expect("GET should succeed")), "hello");
    assert_eq!(int(&handler.del(&k).expect("DEL should succeed")), 1);
}

/// A missing key is nil, and nil is not an empty string.
///
/// These are two different facts about the store, so collapsing them would make a
/// cache-miss indistinguishable from a cached empty value.
#[test]
fn missing_key_is_nil_and_distinct_from_empty_string() {
    let Some(handler) = handler() else { return };
    let absent = key("absent");
    let empty = key("empty");
    handler.del(&absent).expect("DEL should succeed");
    assert_eq!(handler.get(&absent).expect("GET should succeed"), Value::Nil);
    assert_eq!(int(&handler.exists(&absent).expect("EXISTS")), 0);

    handler.set(&empty, b"", None).expect("SET should succeed");
    let stored = handler.get(&empty).expect("GET should succeed");
    assert_ne!(stored, Value::Nil, "an empty string is a present value");
    assert_eq!(text(&stored), "");
    assert_eq!(int(&handler.exists(&empty).expect("EXISTS")), 1);
    handler.del(&empty).expect("DEL should succeed");
}

/// A server-side error must not poison the connection.
///
/// `INCR` on a non-numeric value is answered with a complete, fully drained error
/// reply, so the connection is released rather than discarded and the very next
/// command on the pool must still line up.
#[test]
fn server_error_leaves_connection_usable() {
    let Some(handler) = handler() else { return };
    let k = key("notanumber");
    handler.set(&k, b"abc", None).expect("SET should succeed");

    let failure = handler.incr(&k).expect_err("INCR on 'abc' must fail");
    assert!(
        failure.contains("redis:"),
        "error should be qualified, got {failure}"
    );
    assert!(
        !failure.contains("transport"),
        "a server error is not a transport failure, got {failure}"
    );

    // The reply stream must still be aligned: this GET returns its own answer.
    assert_eq!(text(&handler.get(&k).expect("GET after error")), "abc");
    handler.del(&k).expect("DEL should succeed");
}

/// Sequential commands reuse one connection instead of growing the pool.
#[test]
fn pool_does_not_grow_across_sequential_commands() {
    let Some(handler) = handler() else { return };
    let k = key("poolsize");
    let baseline = handler.pool_size();
    for index in 0..64 {
        handler
            .set(&k, index.to_string().as_bytes(), None)
            .expect("SET should succeed");
        handler.get(&k).expect("GET should succeed");
    }
    assert_eq!(
        handler.pool_size(),
        baseline,
        "sequential commands must reuse the same connection"
    );
    handler.del(&k).expect("DEL should succeed");
}

/// A payload containing CRLF and NUL must survive byte-for-byte.
///
/// RESP is length-prefixed, so only a client that treats replies as text can
/// corrupt this. The assertion is on raw bytes for that reason.
#[test]
fn binary_value_round_trips_byte_for_byte() {
    let Some(handler) = handler() else { return };
    let k = key("binary");
    let payload: Vec<u8> = b"a\r\nb\0c\r\n$3\r\nfake\r\n".to_vec();
    handler.set(&k, &payload, None).expect("SET should succeed");
    let stored = handler.get(&k).expect("GET should succeed");
    assert_eq!(text(&stored).as_bytes(), payload.as_slice());
    // The connection must not have been desynchronized by the embedded frame.
    assert_eq!(int(&handler.exists(&k).expect("EXISTS")), 1);
    handler.del(&k).expect("DEL should succeed");
}

/// `EXPIRE` then `TTL` must report a positive remaining lifetime.
#[test]
fn expire_then_ttl_reports_remaining_time() {
    let Some(handler) = handler() else { return };
    let k = key("ttl");
    handler.set(&k, b"soon", None).expect("SET should succeed");
    assert_eq!(int(&handler.ttl(&k).expect("TTL")), -1, "no expiry yet");
    assert_eq!(int(&handler.expire(&k, 60).expect("EXPIRE")), 1);
    let remaining = int(&handler.ttl(&k).expect("TTL"));
    assert!(
        remaining > 0 && remaining <= 60,
        "TTL should be within (0, 60], got {remaining}"
    );
    handler.del(&k).expect("DEL should succeed");
    assert_eq!(int(&handler.ttl(&k).expect("TTL")), -2, "key is gone");
}

/// `SET` with an expiry and `DECR` are reachable, and `command` is the escape hatch.
#[test]
fn set_with_expiry_decr_and_raw_command() {
    let Some(handler) = handler() else { return };
    let k = key("counter");
    handler.del(&k).expect("DEL should succeed");
    handler.set(&k, b"5", Some(60)).expect("SET EX should succeed");
    assert_eq!(int(&handler.decr(&k).expect("DECR")), 4);
    assert_eq!(int(&handler.incr(&k).expect("INCR")), 5);
    assert!(int(&handler.ttl(&k).expect("TTL")) > 0, "EX must set a TTL");

    let pong = handler
        .command(&[b"PING".to_vec()])
        .expect("PING should succeed");
    assert_eq!(text(&pong), "PONG");
    assert!(handler.command(&[]).is_err(), "empty command is refused");
    handler.del(&k).expect("DEL should succeed");
}
