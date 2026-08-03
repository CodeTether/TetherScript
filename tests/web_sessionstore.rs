//! Behaviour tests for the Redis-backed session-store and durable-rate-limit logic.
//!
//! Every case runs a real `.tether` program through the binary. That is the only
//! way to reach this group: key derivation, escaping, tagging, and the window
//! arithmetic are all private submodules a unit test could not name, and the script
//! surface is what a port actually consumes.
//!
//! No test talks to Redis, and none needs to — that is the point of the split. The
//! rate-limit cases pass `now_secs` explicitly rather than reading a clock, which is
//! what makes an exact window boundary testable at all.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Write `source` to a temp file and run it, returning the raw process output.
fn run_source(source: &str) -> std::process::Output {
    let dir = std::env::temp_dir().join(format!("tether_sessionstore_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, source).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

/// Run a program and return its trimmed stdout, asserting it succeeded.
fn stdout_of(source: &str) -> String {
    let output = run_source(source);
    assert!(
        output.status.success(),
        "script failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .trim_end()
        .to_string()
}

// ---------------------------------------------------------------------------
// Key derivation
// ---------------------------------------------------------------------------

#[test]
fn a_session_key_is_the_prefix_joined_to_the_id() {
    let out = stdout_of(
        r#"
fn main() {
    println(session_store_key("sess", "9f2c").unwrap())
    println(session_store_key("myapp-sess", "abcdef").unwrap())
}
"#,
    );
    assert_eq!(
        out, "sess:9f2c\nmyapp-sess:abcdef",
        "the key must be `prefix:id`, full output: {out}"
    );
}

/// The load-bearing security case: an id arrives in a cookie, so an id carrying the
/// namespace separator would let a request address another namespace's key.
#[test]
fn an_id_containing_the_key_separator_is_rejected_as_key_injection() {
    let out = stdout_of(
        r#"
fn main() {
    let crafted = session_store_key("sess", "x:ratelimit:1.2.3.4:60:0")
    println(str(crafted.is_err()))
    println(crafted.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0], "true",
        "an id containing `:` must not produce a key: {out}"
    );
    assert!(
        lines[1].contains("session_store_key: session_id")
            && lines[1].contains("key injection")
            && lines[1].contains("separator"),
        "the error must name the parameter and the attack, got: {}",
        lines[1]
    );
}

#[test]
fn a_prefix_containing_the_key_separator_is_also_rejected() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = session_store_key("sess:v2", "9f2c")
    println(str(bad.is_err()))
    println(str(bad.err().contains("session_store_key: prefix")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue",
        "a configured prefix is validated too, full output: {out}"
    );
}

#[test]
fn an_empty_or_control_bearing_id_is_rejected() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(session_store_key("sess", "").is_err()))
    println(str(session_store_key("sess", "ab\ncd").is_err()))
    println(str(session_store_key("", "9f2c").is_err()))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "empty and control-bearing components must be refused, full output: {out}"
    );
}

#[test]
fn a_freshly_minted_id_always_survives_key_derivation() {
    let out = stdout_of(
        r#"
fn main() {
    let id = session_store_new_id()
    let key = session_store_key("sess", id)
    println(str(key.is_ok()))
    println(str(key.unwrap() == "sess:" + id))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue",
        "hex ids must never trip their own validator, full output: {out}"
    );
}

// ---------------------------------------------------------------------------
// Encoding round-trip
// ---------------------------------------------------------------------------

#[test]
fn encoding_is_sorted_and_type_tagged() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.uid = 42
    data.admin = true
    data.name = "ada"
    println(session_store_encode(data).unwrap())
}
"#,
    );
    assert_eq!(
        out, "admin=btrue;name=sada;uid=i42",
        "keys must be sorted and values tagged, full output: {out}"
    );
}

/// A naive `split(';')` / `split_once('=')` would tear this entry apart. The escape
/// table is what keeps the round-trip exact.
#[test]
fn a_value_containing_both_separators_round_trips_exactly() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.note = "a;b=c"
    let text = session_store_encode(data).unwrap()
    println(text)
    println(session_store_decode(text).unwrap().note)
}
"#,
    );
    assert_eq!(
        out, "note=sa\\sb\\ec\na;b=c",
        "separators inside a value must be escaped and restored, full output: {out}"
    );
}

#[test]
fn a_key_containing_a_separator_round_trips_exactly() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data["we;ird=key"] = "v"
    let text = session_store_encode(data).unwrap()
    println(text)
    let back = session_store_decode(text).unwrap()
    println(back["we;ird=key"])
}
"#,
    );
    assert_eq!(
        out, "we\\sird\\ekey=sv\nv",
        "both halves of an entry are escaped, full output: {out}"
    );
}

/// A raw newline would break any line-oriented consumer of the encoded form, so it
/// is escaped: the encoded text stays one line and decodes back to a real newline.
#[test]
fn a_value_containing_a_newline_round_trips_and_stays_one_line() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.flash = "saved\nok"
    let text = session_store_encode(data).unwrap()
    println(str(text.lines().len()))
    println(text)
    let back = session_store_decode(text).unwrap().flash
    println(str(back == "saved\nok"))
    println(str(back.lines().len()))
}
"#,
    );
    assert_eq!(
        out, "1\nflash=ssaved\\nok\ntrue\n2",
        "a newline must survive as an escape, full output: {out}"
    );
}

#[test]
fn a_value_containing_a_backslash_round_trips_exactly() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.path = "C:\\tmp"
    let text = session_store_encode(data).unwrap()
    println(text)
    println(str(session_store_decode(text).unwrap().path == "C:\\tmp"))

    // Text that *looks* like an escape must not be mistaken for one.
    let data2 = map()
    data2.escapey = "a\\sb"
    let round = session_store_decode(session_store_encode(data2).unwrap()).unwrap()
    println(str(round.escapey == "a\\sb"))
}
"#,
    );
    // A literal backslash is doubled on the way out, so it cannot be mistaken for the
    // start of an escape on the way back.
    assert_eq!(
        out, "path=sC:\\\\tmp\ntrue\ntrue",
        "a literal backslash must not be able to forge an escape, full output: {out}"
    );
}

#[test]
fn an_empty_map_round_trips_through_the_empty_string() {
    let out = stdout_of(
        r#"
fn main() {
    let text = session_store_encode(map()).unwrap()
    println("[" + text + "]")
    println(str(session_store_decode(text).unwrap().len()))
}
"#,
    );
    assert_eq!(
        out, "[]\n0",
        "an empty payload must encode and decode losslessly, full output: {out}"
    );
}

#[test]
fn every_scalar_type_round_trips_with_its_type_intact() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.i = 7
    data.f = 1.5
    data.b = false
    data.n = nil
    data.s = "7"
    let back = session_store_decode(session_store_encode(data).unwrap()).unwrap()
    println(str(back.i == 7))
    println(str(back.f == 1.5))
    println(str(back.b == false))
    println(str(back.n == nil))
    println(str(back.s == "7"))
    println(str(back.s == 7))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue\ntrue\ntrue\nfalse",
        "a string \"7\" must not come back as an int, full output: {out}"
    );
}

#[test]
fn a_nested_value_is_refused_rather_than_flattened() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.items = ["a", "b"]
    let bad = session_store_encode(data)
    println(str(bad.is_err()))
    println(str(bad.err().contains("session_store_encode")))
    println(str(bad.err().contains("items")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "a list must be reported, not silently stringified, full output: {out}"
    );
}

#[test]
fn malformed_encoded_text_is_reported_by_name() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(session_store_decode("no-separator-here").is_err()))
    println(str(session_store_decode("k=zbogus").is_err()))
    println(str(session_store_decode("k=i12x").is_err()))
    println(str(session_store_decode("k=s\\q").is_err()))
    println(str(session_store_decode("k=sa\\").is_err()))
    println(str(session_store_decode("=sv").is_err()))
    println(str(session_store_decode("k=sa;k=sb").is_err()))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue",
        "every malformed form must fail loudly, full output: {out}"
    );
}

#[test]
fn a_wrong_argument_type_names_the_parameter() {
    let out = stdout_of(
        r#"
fn main() {
    println(session_store_encode("not a map").err())
    println(session_store_decode(7).err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].contains("session_store_encode: payload_map") && lines[0].contains("str"),
        "the encode error must name the parameter and the actual type: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("session_store_decode: text") && lines[1].contains("int"),
        "the decode error must name the parameter and the actual type: {}",
        lines[1]
    );
}

// ---------------------------------------------------------------------------
// Id generation and rotation
// ---------------------------------------------------------------------------

/// 32 CSPRNG bytes as hex: 64 characters, 256 bits. Checked by shape rather than
/// through `hex_decode`, which would fail on bytes that are not valid UTF-8.
#[test]
fn a_new_id_is_sixty_four_lowercase_hex_characters() {
    let out = stdout_of(
        r#"
fn main() {
    let id = session_store_new_id()
    println(str(id.len()))

    let allowed = "0123456789abcdef"
    let mut bad = 0
    let mut i = 0
    while i < id.len() {
        if !allowed.contains(id[i]) { bad = bad + 1 }
        i = i + 1
    }
    println(str(bad))
}
"#,
    );
    assert_eq!(
        out, "64\n0",
        "an id must be 32 bytes of entropy in lowercase hex, full output: {out}"
    );
}

#[test]
fn many_draws_are_all_distinct() {
    let out = stdout_of(
        r#"
fn main() {
    let seen = map()
    let mut i = 0
    while i < 500 {
        seen[session_store_new_id()] = 1
        i = i + 1
    }
    println(str(seen.len()))
}
"#,
    );
    assert_eq!(
        out, "500",
        "500 draws must yield 500 distinct ids, full output: {out}"
    );
}

/// Rotation on privilege change is what defeats session fixation: the id the
/// attacker planted before login must stop naming anything.
#[test]
fn rotation_produces_a_different_id_of_the_same_shape() {
    let out = stdout_of(
        r#"
fn main() {
    let old = session_store_new_id()
    let next = session_rotate_id(old).unwrap()
    println(str(next != old))
    println(str(next.len()))

    let again = session_rotate_id(next).unwrap()
    println(str(again != next))
    println(str(again != old))
}
"#,
    );
    assert_eq!(
        out, "true\n64\ntrue\ntrue",
        "each rotation must yield a fresh id, full output: {out}"
    );
}

#[test]
fn rotating_an_unusable_old_id_is_reported() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(session_rotate_id("a:b").is_err()))
    println(str(session_rotate_id("").is_err()))
    println(str(session_rotate_id(7).err().contains("session_rotate_id: old_id")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "a cookie-supplied old id is validated, full output: {out}"
    );
}

// ---------------------------------------------------------------------------
// Fixed-window bucket key
// ---------------------------------------------------------------------------

#[test]
fn the_window_key_carries_the_subject_window_and_index() {
    let out = stdout_of(
        r#"
fn main() {
    println(ratelimit_window_key("rl", "1.2.3.4", 60, 125).unwrap())
    println(ratelimit_window_key("rl", "1.2.3.4", 60, 0).unwrap())
}
"#,
    );
    assert_eq!(
        out, "rl:1.2.3.4:60:2\nrl:1.2.3.4:60:0",
        "the key must name subject, window, and index, full output: {out}"
    );
}

#[test]
fn the_window_key_is_stable_within_a_window_and_changes_at_the_boundary() {
    let out = stdout_of(
        r#"
fn main() {
    let a = ratelimit_window_key("rl", "u", 60, 120).unwrap()
    let b = ratelimit_window_key("rl", "u", 60, 121).unwrap()
    let c = ratelimit_window_key("rl", "u", 60, 179).unwrap()
    let d = ratelimit_window_key("rl", "u", 60, 180).unwrap()
    println(str(a == b))
    println(str(a == c))
    println(str(a == d))
    println(d)
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\nfalse\nrl:u:60:3",
        "the bucket must roll over exactly at the boundary, full output: {out}"
    );
}

/// Two processes computing the same key is the whole point: the counter is shared,
/// so the limit is not per-worker and does not reset on restart.
#[test]
fn the_window_size_is_part_of_the_key() {
    let out = stdout_of(
        r#"
fn main() {
    let minute = ratelimit_window_key("rl", "u", 60, 600).unwrap()
    let hour = ratelimit_window_key("rl", "u", 3600, 600).unwrap()
    println(str(minute == hour))
    println(minute)
    println(hour)
}
"#,
    );
    assert_eq!(
        out, "false\nrl:u:60:10\nrl:u:3600:0",
        "a reconfigured window must not inherit the old counter, full output: {out}"
    );
}

#[test]
fn a_subject_containing_the_key_separator_is_rejected() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = ratelimit_window_key("rl", "a:b", 60, 0)
    println(str(bad.is_err()))
    println(str(bad.err().contains("ratelimit_window_key: subject")))
    println(str(bad.err().contains("key injection")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "a header-supplied subject is untrusted, full output: {out}"
    );
}

#[test]
fn a_zero_or_negative_window_is_rejected_by_the_key() {
    let out = stdout_of(
        r#"
fn main() {
    let zero = ratelimit_window_key("rl", "u", 0, 100)
    println(str(zero.is_err()))
    println(str(zero.err().contains("window_secs must be a positive")))
    println(str(ratelimit_window_key("rl", "u", -60, 100).is_err()))
    println(str(ratelimit_window_key("rl", "u", 60, -1).is_err()))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue\ntrue",
        "a window of zero would divide by zero, full output: {out}"
    );
}

// ---------------------------------------------------------------------------
// Verdict
// ---------------------------------------------------------------------------

#[test]
fn a_count_of_zero_reports_the_full_allowance() {
    let out = stdout_of(
        r#"
fn main() {
    let v = ratelimit_window_verdict(0, 5, 60, 120).unwrap()
    println(str(v.allowed))
    println(str(v.remaining))
    println(str(v.retry_after_secs))
}
"#,
    );
    assert_eq!(
        out, "true\n5\n0",
        "an uncounted window must allow with everything remaining, full output: {out}"
    );
}

#[test]
fn the_verdict_flips_exactly_between_the_limit_and_the_next_request() {
    let out = stdout_of(
        r#"
fn main() {
    let below = ratelimit_window_verdict(4, 5, 60, 120).unwrap()
    let at = ratelimit_window_verdict(5, 5, 60, 120).unwrap()
    let over = ratelimit_window_verdict(6, 5, 60, 120).unwrap()
    println(str(below.allowed) + " " + str(below.remaining))
    println(str(at.allowed) + " " + str(at.remaining))
    println(str(over.allowed) + " " + str(over.remaining))
}
"#,
    );
    assert_eq!(
        out, "true 1\ntrue 0\nfalse 0",
        "the limit-th request is the last allowed one, full output: {out}"
    );
}

#[test]
fn remaining_is_clamped_at_zero_rather_than_going_negative() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(ratelimit_window_verdict(50, 5, 60, 120).unwrap().remaining))
}
"#,
    );
    assert_eq!(
        out, "0",
        "a negative X-RateLimit-Remaining is not meaningful, full output: {out}"
    );
}

/// `reset_at` is the end of the window containing `now`, so `Retry-After` is
/// truthful: `reset_at - now`, and never zero when denied.
#[test]
fn reset_at_and_retry_after_are_the_window_end_minus_now() {
    let out = stdout_of(
        r#"
fn main() {
    let start = ratelimit_window_verdict(9, 5, 60, 120).unwrap()
    println(str(start.reset_at) + " " + str(start.retry_after_secs))

    let mid = ratelimit_window_verdict(9, 5, 60, 150).unwrap()
    println(str(mid.reset_at) + " " + str(mid.retry_after_secs))

    let last = ratelimit_window_verdict(9, 5, 60, 179).unwrap()
    println(str(last.reset_at) + " " + str(last.retry_after_secs))
}
"#,
    );
    assert_eq!(
        out, "180 60\n180 30\n180 1",
        "retry must equal the time left in the window, full output: {out}"
    );
}

#[test]
fn crossing_the_boundary_moves_reset_at_to_the_next_window_end() {
    let out = stdout_of(
        r#"
fn main() {
    let before = ratelimit_window_verdict(9, 5, 60, 179).unwrap()
    let after = ratelimit_window_verdict(9, 5, 60, 180).unwrap()
    println(str(before.reset_at))
    println(str(after.reset_at))
    println(str(after.retry_after_secs))
}
"#,
    );
    assert_eq!(
        out, "180\n240\n60",
        "reset_at must follow the bucket, full output: {out}"
    );
}

/// A denied caller must never be told to retry immediately: that is a retry storm
/// aimed at an already-saturated server.
#[test]
fn a_denied_verdict_never_reports_a_retry_of_zero() {
    let out = stdout_of(
        r#"
fn main() {
    let mut zeros = 0
    let mut now = 0
    while now < 240 {
        let v = ratelimit_window_verdict(99, 5, 60, now).unwrap()
        if v.retry_after_secs < 1 { zeros = zeros + 1 }
        if v.reset_at <= now { zeros = zeros + 1 }
        now = now + 1
    }
    println(str(zeros))
}
"#,
    );
    assert_eq!(
        out, "0",
        "retry must be at least 1 and reset_at strictly ahead, full output: {out}"
    );
}

/// The honest worst case: a fixed window admits up to 2x the limit across a
/// boundary. Asserted so the documentation cannot drift from the behaviour.
#[test]
fn a_fixed_window_admits_twice_the_limit_across_a_boundary() {
    let out = stdout_of(
        r#"
fn main() {
    let limit = 5
    // Window 1 fully consumed at its last second, then window 2 from zero.
    let mut admitted = 0
    let mut count = 1
    while count <= limit {
        if ratelimit_window_verdict(count, limit, 60, 179).unwrap().allowed {
            admitted = admitted + 1
        }
        count = count + 1
    }
    count = 1
    while count <= limit {
        if ratelimit_window_verdict(count, limit, 60, 180).unwrap().allowed {
            admitted = admitted + 1
        }
        count = count + 1
    }
    println(str(admitted))
}
"#,
    );
    assert_eq!(
        out, "10",
        "two adjacent windows admit 2x the limit within ~1s, full output: {out}"
    );
}

#[test]
fn the_verdict_rejects_out_of_range_arguments_by_name() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(ratelimit_window_verdict(1, 5, 0, 120).err().contains("window_secs")))
    println(str(ratelimit_window_verdict(1, 5, -60, 120).is_err()))
    println(str(ratelimit_window_verdict(-1, 5, 60, 120).err().contains("count")))
    println(str(ratelimit_window_verdict(1, 0, 60, 120).err().contains("limit")))
    println(str(ratelimit_window_verdict(1, 5, 60, -1).err().contains("now_secs")))
    println(str(ratelimit_window_verdict("1", 5, 60, 120).err().contains("must be int")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue\ntrue\ntrue\ntrue",
        "every bad argument must name itself, full output: {out}"
    );
}

// ---------------------------------------------------------------------------
// The two halves used together
// ---------------------------------------------------------------------------

/// The shape a handler actually uses: mint, key, encode, rotate on login, re-key.
#[test]
fn the_session_half_composes_into_a_store_and_rotate_flow() {
    let out = stdout_of(
        r#"
fn main() {
    let id = session_store_new_id()
    let key = session_store_key("sess", id).unwrap()

    let data = map()
    data.uid = 42
    data.role = "anonymous"
    let blob = session_store_encode(data).unwrap()

    // ... the Redis owner would SETEX(key, ttl, blob) here.
    let loaded = session_store_decode(blob).unwrap()
    println(str(loaded.uid))
    println(loaded.role)

    let next = session_rotate_id(id).unwrap()
    let next_key = session_store_key("sess", next).unwrap()
    println(str(next_key != key))
    println(str(next_key == "sess:" + next))
}
"#,
    );
    assert_eq!(
        out, "42\nanonymous\ntrue\ntrue",
        "the pieces must compose without a transport, full output: {out}"
    );
}
