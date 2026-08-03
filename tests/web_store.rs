//! Behaviour tests for the server-side session store.
//!
//! Every case runs a real `.tether` program through the binary, because the store
//! is only reachable through the registered built-ins: the backend, the record, and
//! the expiry policy are all private submodules a unit test could not see, and the
//! script surface is what a port actually consumes.
//!
//! Two conventions carry weight here. Ids are never invented — a "valid-looking"
//! id is always one the store really minted, which is how an attacker holding a
//! stale cookie would present it. And the timing cases use a deliberately tiny TTL
//! plus `sleep_ms` rather than a mocked clock, because the clock is
//! `time_now_ms` and there is no script-visible way to move it.
//!
//! The `format!` cases escape literal braces as `{{` / `}}`, matching the
//! convention in the other `tests/web_*.rs` harnesses.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a program and return its trimmed stdout, asserting it succeeded.
fn stdout_of(src: &str) -> String {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_store_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("store_case_{case}.tether"));
    std::fs::write(&path, src).expect("source should be writable");
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
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

#[test]
fn create_then_load_round_trips_the_payload() {
    let out = stdout_of(
        r#"
fn main() {
    let data = map()
    data.cart = "sku-7"
    data.locale = "en-GB"
    let created = store_create("user-1", data).unwrap()

    let loaded = store_load(created.id).unwrap()
    println(loaded.data.cart)
    println(loaded.data.locale)
    println(loaded.subject)
    println(str(loaded.id == created.id))
}
"#,
    );
    assert_eq!(
        out, "sku-7\nen-GB\nuser-1\ntrue",
        "a created session must load back intact, full output: {out}"
    );
}

/// The id is 32 CSPRNG bytes as hex: 64 characters, 256 bits. Checked by shape
/// rather than through `hex_decode`, which rejects bytes that are not valid UTF-8
/// and so would fail on most legitimate ids.
#[test]
fn a_generated_id_is_sixty_four_lowercase_hex_characters() {
    let out = stdout_of(
        r#"
fn main() {
    let s = store_create("user-1", map()).unwrap()
    println(str(s.id.len()))

    let allowed = "0123456789abcdef"
    let mut bad = 0
    let mut i = 0
    while i < s.id.len() {
        if !allowed.contains(s.id[i]) { bad = bad + 1 }
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
fn loading_an_unknown_id_fails_by_name() {
    let out = stdout_of(
        r#"
fn main() {
    // Never minted by the store, and the right shape, so this is not a type error.
    let bad = store_load("0000000000000000000000000000000000000000000000000000000000000000")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "an unknown id must not load: {out}");
    assert!(
        lines[1].contains("store_load") && lines[1].contains("no session with that id"),
        "the error must name the call and the cause, got: {}",
        lines[1]
    );
}

#[test]
fn a_non_string_id_is_rejected_by_type() {
    let out = stdout_of(
        r#"
fn main() {
    let bad = store_load(42)
    println(bad.err())
}
"#,
    );
    assert!(
        out.contains("store_load: id must be str, got int"),
        "the error must name the parameter and both types, got: {out}"
    );
}

/// Idle timeout: an abandoned session must stop working even though its absolute
/// ceiling is far away.
#[test]
fn the_idle_timeout_expires_an_untouched_session() {
    let out = stdout_of(
        r#"
fn main() {
    // 40ms idle, 1 hour absolute: only the idle clock can fire here.
    store_configure(40, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()
    println(str(store_load(s.id).is_ok()))

    sleep_ms(120).unwrap()
    let stale = store_load(s.id)
    println(str(stale.is_err()))
    println(stale.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "a fresh session must load: {out}");
    assert_eq!(lines[1], "true", "an idle session must not load: {out}");
    assert!(
        lines[2].contains("idle timeout"),
        "the error must say which clock fired, got: {}",
        lines[2]
    );
}

/// Absolute lifetime: a *stolen* session stays busy, so the idle clock never fires.
/// Only the ceiling can end it, which is why one clock alone is insufficient.
#[test]
fn the_absolute_lifetime_expires_a_session_that_keeps_being_touched() {
    let out = stdout_of(
        r#"
fn main() {
    // 10 minute idle so it can never fire; 120ms ceiling.
    store_configure(600000, 120).unwrap()
    let s = store_create("user-1", map()).unwrap()

    let mut touches = 0
    let mut i = 0
    while i < 8 {
        sleep_ms(30).unwrap()
        let t = store_touch(s.id)
        if t.is_ok() { touches = touches + 1 }
        i = i + 1
    }

    // Activity kept the idle clock reset every 30ms, yet the session is gone.
    println(str(touches < 8))
    let dead = store_load(s.id)
    println(str(dead.is_err()))
    println(dead.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0], "true",
        "constant activity must not outlive the ceiling: {out}"
    );
    assert_eq!(lines[1], "true", "the session must be unusable: {out}");
    assert!(
        lines[2].contains("absolute lifetime"),
        "the error must blame the ceiling, not the idle clock, got: {}",
        lines[2]
    );
}

/// `touch` moves `seen_ms` and must leave `created_ms` alone. If it moved both, a
/// stolen id could be kept alive forever and the ceiling would never bind.
#[test]
fn touch_extends_the_idle_clock_but_not_the_absolute_one() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(60000, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()
    sleep_ms(30).unwrap()
    let t = store_touch(s.id).unwrap()

    println(str(t.seen_ms > s.seen_ms))
    println(str(t.created_ms == s.created_ms))
    println(str(t.id == s.id))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "touch must move only the idle clock and keep the id, full output: {out}"
    );
}

/// A session already past its idle window must not be revivable by touching it.
#[test]
fn touching_an_expired_session_fails_rather_than_resurrecting_it() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(40, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()
    sleep_ms(120).unwrap()

    let revived = store_touch(s.id)
    println(str(revived.is_err()))
    println(str(store_load(s.id).is_err()))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue",
        "an expired session must stay expired, full output: {out}"
    );
}

/// Session fixation: the id an attacker planted must stop resolving after login,
/// while the data the user built up before logging in survives.
#[test]
fn rotation_changes_the_id_and_preserves_the_data() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(60000, 3600000).unwrap()
    let pre = map()
    pre.cart = "sku-7"
    pre.csrf = "tok-9"
    let planted = store_create("anonymous", pre).unwrap()

    // The privilege change: log in, rotate.
    let after = store_rotate(planted.id).unwrap()

    println(str(after.id != planted.id))
    println(after.data.cart)
    println(after.data.csrf)

    // The attacker's copy of the old id is now worthless.
    let attacker = store_load(planted.id)
    println(str(attacker.is_err()))
    println(attacker.err())
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "rotation must change the id: {out}");
    assert_eq!(lines[1], "sku-7", "pre-login data must survive: {out}");
    assert_eq!(lines[2], "tok-9", "pre-login data must survive: {out}");
    assert_eq!(lines[3], "true", "the planted id must stop working: {out}");
    assert!(
        lines[4].contains("no session with that id"),
        "the old id must look simply unknown, got: {}",
        lines[4]
    );
}

/// Rotation must not leave a duplicate behind: exactly one record survives it.
#[test]
fn rotation_leaves_exactly_one_record() {
    let out = stdout_of(
        r#"
fn main() {
    store_sweep().unwrap()
    let before = store_count().unwrap()
    let s = store_create("user-1", map()).unwrap()
    let rotated = store_rotate(s.id).unwrap()
    println(str(store_count().unwrap() - before))
    println(str(store_load(rotated.id).is_ok()))
}
"#,
    );
    assert_eq!(
        out, "1\ntrue",
        "rotation must replace rather than duplicate, full output: {out}"
    );
}

/// Revocation is the entire reason a server-side store exists: a cookie that is
/// still perfectly well-formed and nowhere near expiry must become useless.
#[test]
fn destroy_makes_a_valid_looking_unexpired_id_unusable() {
    let out = stdout_of(
        r#"
fn main() {
    // A 7-day ceiling and a 30-minute idle window, so nothing here expires on its
    // own: whatever stops working stopped because it was revoked.
    store_configure(1800000, 604800000).unwrap()
    let s = store_create("user-1", map()).unwrap()

    // The cookie carries the id and nothing else, so this is exactly what a
    // client would still be holding on the next request.
    let cookie = s.id
    println(str(store_load(cookie).is_ok()))

    println(str(store_destroy(s.id).unwrap()))
    let revoked = store_load(cookie)
    println(str(revoked.is_err()))
    println(revoked.err())

    // Logout is idempotent: a second destroy is false, not an error.
    println(str(store_destroy(s.id).unwrap()))
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "the id works before revocation: {out}");
    assert_eq!(lines[1], "true", "destroy must report the removal: {out}");
    assert_eq!(
        lines[2], "true",
        "a revoked but unexpired id must be useless: {out}"
    );
    assert!(
        lines[3].contains("no session with that id"),
        "a revoked id must be indistinguishable from an unknown one, got: {}",
        lines[3]
    );
    assert_eq!(
        lines[4], "false",
        "a second destroy must be idempotent, not an error: {out}"
    );
}

/// Destroying an unknown id is not an error, so a retried logout cannot 500.
#[test]
fn destroying_an_unknown_id_is_false_not_an_error() {
    let out = stdout_of(
        r#"
fn main() {
    let id = "1111111111111111111111111111111111111111111111111111111111111111"
    let gone = store_destroy(id)
    println(str(gone.is_ok()))
    println(str(gone.unwrap()))
}
"#,
    );
    assert_eq!(
        out, "true\nfalse",
        "logout must be idempotent, full output: {out}"
    );
}

/// "Log out everywhere", and the post-password-change sweep.
#[test]
fn destroy_subject_clears_several_sessions_and_spares_others() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(600000, 3600000).unwrap()
    let a = store_create("user-1", map()).unwrap()
    let b = store_create("user-1", map()).unwrap()
    let c = store_create("user-1", map()).unwrap()
    let other = store_create("user-2", map()).unwrap()

    println(str(store_destroy_subject("user-1").unwrap()))
    println(str(store_load(a.id).is_err()))
    println(str(store_load(b.id).is_err()))
    println(str(store_load(c.id).is_err()))
    println(str(store_load(other.id).is_ok()))

    // A subject with nothing left removes zero, which is a normal answer.
    println(str(store_destroy_subject("user-1").unwrap()))
}
"#,
    );
    assert_eq!(
        out, "3\ntrue\ntrue\ntrue\ntrue\n0",
        "the sweep must clear one subject and only that subject, full output: {out}"
    );
}

/// Colliding ids would hand one user another user's session, so this is the single
/// most consequential property of the generator.
#[test]
fn ids_never_collide_across_many_generations() {
    let generations = 500;
    let src = format!(
        r#"
fn main() {{
    store_configure(600000, 3600000).unwrap()
    let seen = map()
    let mut collisions = 0
    let mut minted = 0
    let mut i = 0
    while i < {generations} {{
        let s = store_create("user-1", map()).unwrap()
        if seen.contains(s.id) {{ collisions = collisions + 1 }}
        seen[s.id] = true
        minted = minted + 1
        i = i + 1
    }}
    println(str(minted))
    println(str(seen.len()))
    println(str(collisions))
}}
"#
    );
    let out = stdout_of(&src);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0],
        generations.to_string(),
        "every create must succeed: {out}"
    );
    assert_eq!(
        lines[1],
        generations.to_string(),
        "{generations} ids must yield {generations} distinct keys: {out}"
    );
    assert_eq!(lines[2], "0", "no two ids may ever collide: {out}");
}

/// Rotation draws from the same generator, so its ids must be distinct too.
#[test]
fn rotated_ids_never_repeat_across_many_rotations() {
    let src = r#"
fn main() {
    store_configure(600000, 3600000).unwrap()
    let seen = map()
    let mut id = store_create("user-1", map()).unwrap().id
    seen[id] = true

    let mut i = 0
    while i < 200 {
        let next = store_rotate(id).unwrap()
        if seen.contains(next.id) { println("collision") }
        seen[next.id] = true
        id = next.id
        i = i + 1
    }
    println(str(seen.len()))
}
"#;
    let out = stdout_of(src);
    assert_eq!(
        out, "201",
        "201 minted ids must all be distinct, full output: {out}"
    );
}

/// Saving replaces the payload rather than merging it, so a cleared claim is gone.
#[test]
fn save_replaces_the_payload_so_a_dropped_key_is_really_dropped() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(600000, 3600000).unwrap()
    let first = map()
    first.role = "admin"
    first.cart = "sku-7"
    let s = store_create("user-1", first).unwrap()

    let downgraded = map()
    downgraded.cart = "sku-7"
    store_save(s.id, downgraded).unwrap()

    let loaded = store_load(s.id).unwrap()
    println(str(loaded.data.contains("role")))
    println(loaded.data.cart)
}
"#,
    );
    assert_eq!(
        out, "false\nsku-7",
        "an omitted key must not survive a save, full output: {out}"
    );
}

/// The map a script gets back is a copy, so editing it must not reach the store.
#[test]
fn editing_a_loaded_session_does_not_reach_the_store_without_save() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(600000, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()

    let loaded = store_load(s.id).unwrap()
    loaded.data.role = "admin"

    let again = store_load(s.id).unwrap()
    println(str(again.data.contains("role")))
}
"#,
    );
    assert_eq!(
        out, "false",
        "a returned payload must be a copy, full output: {out}"
    );
}

/// Sweeping reclaims space and must not change any answer a script can observe.
#[test]
fn sweep_drops_expired_records_without_changing_any_answer() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(40, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()
    sleep_ms(120).unwrap()

    // Already unusable before the sweep: expiry is enforced on read.
    println(str(store_load(s.id).is_err()))
    println(str(store_sweep().unwrap() > 0))
    println(str(store_load(s.id).is_err()))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue\ntrue",
        "sweeping must be observationally neutral, full output: {out}"
    );
}

/// A negative TTL is meaningless, so it must be named rather than accepted. A
/// ceiling *shorter* than the idle window is deliberately legal: "a hard five
/// minutes regardless of activity" is a real policy.
#[test]
fn configure_rejects_a_negative_ttl_but_allows_a_short_ceiling() {
    let out = stdout_of(
        r#"
fn main() {
    let negative = store_configure(1000, -1)
    println(str(negative.is_err()))
    println(negative.err())
    println(str(store_configure(-5, 1000).is_err()))
    println(str(store_configure(600000, 1000).is_ok()))
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "a negative ceiling must error: {out}");
    assert!(
        lines[1].contains("absolute_ttl_ms") && lines[1].contains("must not be negative"),
        "the error must name the offending parameter, got: {}",
        lines[1]
    );
    assert_eq!(lines[2], "true", "a negative idle window must error: {out}");
    assert_eq!(
        lines[3], "true",
        "a ceiling below the idle window is a legitimate policy: {out}"
    );
}

/// Setting a clock to zero disables it, which is how a caller opts out of one rule
/// while keeping the other.
#[test]
fn a_zero_ttl_disables_that_clock() {
    let out = stdout_of(
        r#"
fn main() {
    // No idle window at all; only the ceiling applies.
    store_configure(0, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()
    sleep_ms(120).unwrap()
    println(str(store_load(s.id).is_ok()))
}
"#,
    );
    assert_eq!(
        out, "true",
        "a zero idle window must not expire anything, full output: {out}"
    );
}

/// TTLs are captured at creation, so a later policy change cannot retroactively
/// lengthen a session an operator already believes is bounded.
#[test]
fn a_live_session_keeps_the_ttls_it_was_created_with() {
    let out = stdout_of(
        r#"
fn main() {
    store_configure(40, 3600000).unwrap()
    let s = store_create("user-1", map()).unwrap()

    // Widen the policy after the fact; the live session must not benefit.
    store_configure(600000, 3600000).unwrap()
    sleep_ms(120).unwrap()

    let stale = store_load(s.id)
    println(str(stale.is_err()))
    println(str(store_create("user-2", map()).unwrap().idle_ttl_ms))
}
"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0], "true",
        "the old session must still expire on its own TTL: {out}"
    );
    assert_eq!(
        lines[1], "600000",
        "a new session must pick up the new policy: {out}"
    );
}

/// The id is safe in a cookie exactly as minted, so no caller has to re-encode it.
#[test]
fn a_session_id_serializes_into_a_cookie_unchanged() {
    let out = stdout_of(
        r#"
fn main() {
    let s = store_create("user-1", map()).unwrap()
    let opts = map()
    opts.http_only = true
    opts.path = "/"
    let header = cookie_serialize("sid", s.id, opts).unwrap()
    println(str(header.contains(s.id)))
    println(str(header.contains("HttpOnly")))
}
"#,
    );
    assert_eq!(
        out, "true\ntrue",
        "a hex id must pass the cookie guard untouched, full output: {out}"
    );
}

/// The store is process-local, a documented limitation of the in-memory backend:
/// a second process must not see the first one's sessions.
#[test]
fn the_in_memory_backend_is_not_shared_across_processes() {
    let first = stdout_of(
        r#"
fn main() {
    store_configure(600000, 3600000).unwrap()
    println(store_create("user-1", map()).unwrap().id)
}
"#,
    );
    let id = first.trim();
    assert_eq!(id.len(), 64, "expected an id, got: {first}");
    let out = stdout_of(&format!(
        r#"
fn main() {{
    let carried = store_load("{id}")
    println(str(carried.is_err()))
    println(str(store_count().unwrap()))
}}
"#
    ));
    assert_eq!(
        out, "true\n0",
        "a fresh process must start with an empty store, full output: {out}"
    );
}
