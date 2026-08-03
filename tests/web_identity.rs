//! Coverage for the identity and request-context built-ins.
//!
//! These run real `.tether` programs through the binary, because every concern
//! module in the group is private and the script surface is what a handler actually
//! consumes.
//!
//! The security-relevant cases carry the most weight and are asserted explicitly:
//!
//! * A caller-supplied `X-Request-ID` containing a newline or an ANSI escape is
//!   **replaced**, not sanitised, because it lands in logs where a newline forges an
//!   entry and an escape rewrites an operator's screen.
//! * `identity_from_claims` never yields `authenticated: true` from nothing.
//! * `has_role` matches whole strings, so `admin` is neither found in
//!   `administrator-readonly` (prefix) nor in `not-admin` (substring).
//! * A `roles` claim that is a bare string is refused rather than wrapped.
//! * `require_role` answers 403, never 401.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script, returning the completed process without asserting success.
fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_identity_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("identity_case_{case}.tether"));
    std::fs::write(&path, src).expect("source should be writable");
    Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run")
}

/// Run a program and return its trimmed stdout, asserting it succeeded.
fn stdout_of(src: &str) -> String {
    let output = run_source(src);
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

/// Shared prelude building a realistic request map behind a proxy.
const REALISTIC_REQUEST: &str = r#"
fn request() {
    let h = map()
    h["Host"] = "shop.example.com"
    h["User-Agent"] = "Mozilla/5.0 (X11; Linux x86_64)"
    h["Referer"] = "https://shop.example.com/cart"
    h["X-Forwarded-For"] = "203.0.113.7, 10.0.0.1"
    h["X-Forwarded-Proto"] = "https"
    h["X-Request-ID"] = "req-0000-abc_DEF-42"
    let r = map()
    r["method"] = "POST"
    r["path"] = "/checkout"
    r["query"] = "step=2"
    r["headers"] = h
    r["body"] = "payload"
    r["remote_addr"] = "10.0.0.1"
    r
}
"#;

#[test]
fn a_realistic_request_yields_every_context_field() {
    let out = stdout_of(&format!(
        r#"{REALISTIC_REQUEST}
fn main() {{
    let ctx = request_context(request())?
    println(ctx.method)
    println(ctx.path)
    println(ctx.query)
    println(ctx.client_ip)
    println(ctx.user_agent)
    println(ctx.referer)
    println(ctx.request_id)
    println(str(ctx.is_secure))
}}"#
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "POST");
    assert_eq!(lines[1], "/checkout");
    assert_eq!(lines[2], "step=2");
    // Leftmost X-Forwarded-For entry: each proxy appends, so the client is first.
    assert_eq!(
        lines[3], "203.0.113.7",
        "client_ip must be the leftmost forwarded entry, not the proxy: {out}"
    );
    assert!(lines[4].starts_with("Mozilla/5.0"), "got: {}", lines[4]);
    assert_eq!(lines[5], "https://shop.example.com/cart");
    assert_eq!(lines[6], "req-0000-abc_DEF-42");
    assert_eq!(lines[7], "true");
}

#[test]
fn a_request_with_no_optional_headers_still_yields_a_context() {
    // A bare request must not error: absent optional headers are ordinary, and a
    // handler that cannot get a context cannot log the request it is refusing.
    let out = stdout_of(
        r#"
fn main() {
    let r = map()
    r["method"] = "GET"
    r["path"] = "/health"
    r["query"] = ""
    r["headers"] = map()
    let ctx = request_context(r)?
    println(ctx.method + " " + ctx.path)
    println(str(ctx.user_agent))
    println(str(ctx.referer))
    println(str(ctx.client_ip == ""))
    println(str(ctx.is_secure))
    println(str(ctx.request_id.len() > 0))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["GET /health", "nil", "nil", "true", "false", "true"],
        "full output: {out}"
    );
}

#[test]
fn a_well_formed_incoming_request_id_is_echoed() {
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["x-request-id"] = "0191d4c2-7f3a-4b1e-9c8d-aa11bb22cc33"
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = h
    println(request_id(r)?)
    println(request_context(r)?.request_id)
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "0191d4c2-7f3a-4b1e-9c8d-aa11bb22cc33");
    assert_eq!(
        lines[1], lines[0],
        "the context and the standalone builtin must agree: {out}"
    );
}

#[test]
fn an_incoming_request_id_containing_a_newline_is_replaced() {
    // A newline splits one log line into two, so the attacker authors the second
    // one. The value must be replaced outright, not trimmed at the newline.
    let out = stdout_of(
        r#"
fn main() {
    let h = map()
    h["x-request-id"] = "ok-part\nfake INFO authorized user=root"
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = h
    let id = request_id(r)?
    println(str(id.contains("\n")))
    println(str(id.contains("fake")))
    println(str(id.contains("ok-part")))
    println(str(id.len()))
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "no newline may survive: {out}");
    assert_eq!(lines[1], "false", "no injected text may survive: {out}");
    assert_eq!(
        lines[2], "false",
        "the safe prefix must not be kept either: a truncated attacker-chosen id is \
         still attacker-influenced, so the value is replaced, not sanitised: {out}"
    );
    assert_eq!(lines[3], "36", "the replacement is a canonical UUID: {out}");
}

#[test]
fn an_incoming_request_id_containing_an_escape_sequence_is_replaced() {
    // "G1sySg==" decodes to ESC [ 2 J — clear screen. Injected into a log, it
    // erases what an operator has already seen.
    let out = stdout_of(
        r#"
fn main() {
    let esc = base64_decode("G1sySg==")?
    let h = map()
    h["x-request-id"] = "abc" + esc + "def"
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = h
    let id = request_id(r)?
    println(str(id.contains(esc)))
    println(str(id.contains("abc")))
    println(str(id.len()))
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "false", "no escape byte may survive: {out}");
    assert_eq!(lines[1], "false", "the value is replaced wholesale: {out}");
    assert_eq!(lines[2], "36", "the replacement is a canonical UUID: {out}");
}

#[test]
fn an_overlong_incoming_request_id_is_replaced() {
    // A caller must not be able to append a kilobyte to every line of the log.
    let out = stdout_of(
        r#"
fn main() {
    let mut long = ""
    let mut i = 0
    while i < 60 {
        long = long + "0123456789"
        i = i + 1
    }
    let h = map()
    h["x-request-id"] = long
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = h
    let id = request_id(r)?
    println(str(long.len()))
    println(str(id.len()))
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "600", "the fixture must exceed the bound: {out}");
    assert_eq!(
        lines[1], "36",
        "an id past the length bound must be replaced, not truncated: {out}"
    );
}

#[test]
fn an_incoming_request_id_with_a_disallowed_character_is_replaced() {
    // The charset is an allowlist: ASCII alphanumerics plus `-` and `_`. A space,
    // a bracket, or a Unicode line separator are all outside it.
    let out = stdout_of(
        r#"
fn main() {
    let bad = ["has space", "brackets[]", "semi;colon", "", "quote\"mark", "tab\there"]
    for candidate in bad {
        let h = map()
        h["x-request-id"] = candidate
        let r = map()
        r["method"] = "GET"
        r["path"] = "/"
        r["query"] = ""
        r["headers"] = h
        println(str(request_id(r)?.len()))
    }
    let good = map()
    good["x-request-id"] = "Aa0-_zZ9"
    let ok = map()
    ok["method"] = "GET"
    ok["path"] = "/"
    ok["query"] = ""
    ok["headers"] = good
    println(request_id(ok)?)
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    for (index, line) in lines[..6].iter().enumerate() {
        assert_eq!(
            *line, "36",
            "candidate {index} should have been replaced: {out}"
        );
    }
    assert_eq!(
        lines[6], "Aa0-_zZ9",
        "the allowed charset must still be echoed: {out}"
    );
}

#[test]
fn two_generated_request_ids_are_distinct() {
    // A correlation id that repeats correlates unrelated requests.
    let out = stdout_of(
        r#"
fn main() {
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = map()
    let first = request_id(r)?
    let second = request_id(r)?
    println(str(first != second))
    println(str(first.len()))
    println(str(second.len()))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "36", "36"],
        "generated ids must be distinct and canonical: {out}"
    );
}

#[test]
fn absent_or_empty_claims_yield_an_anonymous_identity() {
    // The default must be anonymous, so a missing check fails closed.
    let out = stdout_of(
        r#"
fn main() {
    let from_nil = identity_from_claims(nil)?
    let from_empty = identity_from_claims(map())?
    let bare = anonymous()
    println(str(from_nil.authenticated))
    println(str(from_empty.authenticated))
    println(str(bare.authenticated))
    println(str(bare.subject))
    println(str(bare.roles.len()))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false", "false", "nil", "0"],
        "nothing must produce an authenticated identity: {out}"
    );
}

#[test]
fn a_forged_authenticated_claim_does_not_authenticate() {
    // `authenticated` is derived from the subject, never copied out of the claims,
    // so a caller who mints the field gains nothing.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["authenticated"] = true
    claims["roles"] = ["admin"]
    let who = identity_from_claims(claims)?
    println(str(who.authenticated))
    println(str(who.subject))
    println(str(has_role(who, "admin")))

    let blank = map()
    blank["sub"] = ""
    println(str(identity_from_claims(blank)?.authenticated))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "nil", "false", "false"],
        "a subjectless claim set must stay anonymous and roleless: {out}"
    );
}

#[test]
fn claims_with_a_subject_and_roles_yield_an_authenticated_identity() {
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "user-4711"
    claims["roles"] = ["editor", "admin"]
    let who = identity_from_claims(claims)?
    println(str(who.authenticated))
    println(who.subject)
    println(str(who.roles.len()))
    println(str(has_role(who, "admin")))
    println(str(has_role(who, "editor")))
    println(str(has_role(who, "owner")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "user-4711", "2", "true", "true", "false"],
        "full output: {out}"
    );
}

#[test]
fn has_role_matches_exactly_and_not_by_prefix() {
    // `admin` must not match `administrator-readonly`: a deliberately narrowed role
    // must never become a superset of the privileged one.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = ["administrator-readonly"]
    let who = identity_from_claims(claims)?
    println(str(has_role(who, "admin")))
    println(str(has_role(who, "administrator")))
    println(str(has_role(who, "administrator-readonly")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false", "true"],
        "a prefix must never satisfy a role check: {out}"
    );
}

#[test]
fn has_role_does_not_match_a_substring() {
    // `admin` inside `not-admin` must not satisfy the check, or any naming
    // convention that encodes a negation becomes an authorisation bypass.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = ["not-admin", "admin-denied", "pending-admin-approval"]
    let who = identity_from_claims(claims)?
    println(str(has_role(who, "admin")))
    println(str(has_role(who, "denied")))
    println(str(has_role(who, "not-admin")))
    println(str(has_role(who, "Admin")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false", "true", "false"],
        "matching must be whole-string and case-sensitive: {out}"
    );
}

#[test]
fn has_role_is_false_for_an_unauthenticated_identity() {
    let out = stdout_of(
        r#"
fn main() {
    println(str(has_role(anonymous(), "admin")))
    println(str(has_role(map(), "admin")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false"],
        "a caller cannot hold a role without being someone: {out}"
    );
}

#[test]
fn a_string_roles_claim_is_rejected() {
    // A single string must not be silently treated as a one-element list: layers
    // disagree about whether it means one role or its characters, and a value like
    // "admin,editor" would grant one nonexistent role while looking accepted.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = "admin"
    let attempt = identity_from_claims(claims)
    println(str(attempt.is_err()))
    println(attempt.err())

    let csv = map()
    csv["sub"] = "u1"
    csv["roles"] = "admin,editor"
    println(str(identity_from_claims(csv).is_err()))
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "a str roles claim must error: {out}");
    assert!(
        lines[1].contains("must be a list of str"),
        "the error must name the required shape, got: {}",
        lines[1]
    );
    assert_eq!(lines[2], "true", "a CSV roles claim must error too: {out}");
}

#[test]
fn a_non_string_roles_element_is_rejected() {
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = ["admin", 7]
    let attempt = identity_from_claims(claims)
    println(str(attempt.is_err()))
    println(attempt.err())
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true");
    assert!(
        lines[1].contains("entry must be str") && lines[1].contains("int"),
        "the error must name the offending type, got: {}",
        lines[1]
    );
}

#[test]
fn an_absent_roles_claim_is_an_empty_role_list() {
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    let who = identity_from_claims(claims)?
    println(str(who.authenticated))
    println(str(who.roles.len()))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "0"],
        "a caller with no roles is normal, not an error: {out}"
    );
}

#[test]
fn require_role_returns_nil_when_the_role_is_held() {
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = ["admin"]
    let who = identity_from_claims(claims)?
    let denied = require_role(who, "admin")?
    println(str(denied == nil))
}"#,
    );
    assert_eq!(
        out, "true",
        "holding the role must return nil so the handler continues: {out}"
    );
}

#[test]
fn require_role_returns_403_not_401_when_the_role_is_missing() {
    // 401 means "I do not know who you are" and invites a retry with a credential.
    // This caller is known and will never be permitted, so retrying is pointless:
    // the answer is 403.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = "u1"
    claims["roles"] = ["editor"]
    let who = identity_from_claims(claims)?
    let denied = require_role(who, "admin")?
    println(str(denied != nil))
    println(str(denied.status))
    println(str(denied.status == 401))
    println(denied.body)
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "a missing role must produce a response");
    assert_eq!(
        lines[1], "403",
        "authenticated-but-not-permitted is 403: {out}"
    );
    assert_eq!(
        lines[2], "false",
        "401 would loop a client that already has a valid credential: {out}"
    );
    assert!(
        lines[3].contains("admin"),
        "the body must name the required role, got: {}",
        lines[3]
    );
    assert!(
        !lines[3].contains("editor"),
        "the body must not echo the roles held; that tells an attacker what to \
         target: {}",
        lines[3]
    );
}

#[test]
fn require_role_names_an_unauthenticated_caller_as_an_error() {
    // Not 403: answering 403 to an unauthenticated caller hides that a credential
    // would have helped. The handler must choose between a 401 challenge and
    // anonymous access.
    let out = stdout_of(
        r#"
fn main() {
    let attempt = require_role(anonymous(), "admin")
    println(str(attempt.is_err()))
    println(attempt.err())
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true");
    assert!(
        lines[1].contains("not authenticated") && lines[1].contains("401"),
        "the error must explain the 401/403 distinction, got: {}",
        lines[1]
    );
}

#[test]
fn is_secure_reads_x_forwarded_proto() {
    let out = stdout_of(
        r#"
fn ctx_for(headers) {
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = headers
    request_context(r)
}

fn main() {
    let https = map()
    https["X-Forwarded-Proto"] = "https"
    println(str(ctx_for(https)?.is_secure))

    let cased = map()
    cased["x-forwarded-proto"] = "HTTPS"
    println(str(ctx_for(cased)?.is_secure))

    let chained = map()
    chained["x-forwarded-proto"] = "https, http"
    println(str(ctx_for(chained)?.is_secure))

    let plain = map()
    plain["x-forwarded-proto"] = "http"
    println(str(ctx_for(plain)?.is_secure))

    let ssl = map()
    ssl["x-forwarded-ssl"] = "on"
    println(str(ctx_for(ssl)?.is_secure))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "true", "true", "false", "true"],
        "the leftmost proto entry decides, case-insensitively: {out}"
    );
}

#[test]
fn is_secure_is_false_when_no_proto_header_is_present() {
    // Fail closed: an absent proxy header is plaintext, not optimistically TLS.
    // Note the header is client-controlled and only trustworthy behind a proxy
    // that overwrites it — a direct caller can claim HTTPS over plaintext.
    let out = stdout_of(
        r#"
fn main() {
    let r = map()
    r["method"] = "GET"
    r["path"] = "/"
    r["query"] = ""
    r["headers"] = map()
    println(str(request_context(r)?.is_secure))
}"#,
    );
    assert_eq!(out, "false", "no proto header must mean not secure: {out}");
}

#[test]
fn ip_changed_reports_a_different_address() {
    // A signal to log or re-authenticate on, never a hard failure: IP rotation is
    // routine on mobile networks and CGNAT.
    let out = stdout_of(
        r#"
fn main() {
    let session = map()
    session["client_ip"] = "203.0.113.7"
    println(str(ip_changed(session, "198.51.100.22")))
    println(str(ip_changed(session, "203.0.113.7")))

    let legacy = map()
    legacy["created_ip"] = "203.0.113.7"
    println(str(ip_changed(legacy, "198.51.100.22")))
    println(str(ip_changed(legacy, "203.0.113.7")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "false", "true", "false"],
        "full output: {out}"
    );
}

#[test]
fn ip_changed_is_false_when_either_address_is_unknown() {
    // An unknown address is not evidence of a change; reporting one would make
    // every session with no recorded address look stolen on its first request.
    let out = stdout_of(
        r#"
fn main() {
    println(str(ip_changed(map(), "198.51.100.22")))

    let blank = map()
    blank["client_ip"] = ""
    println(str(ip_changed(blank, "198.51.100.22")))

    let known = map()
    known["client_ip"] = "203.0.113.7"
    println(str(ip_changed(known, "")))
}"#,
    );
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "false", "false"],
        "an unknown address must not raise the signal: {out}"
    );
}

#[test]
fn a_context_feeds_the_session_ip_tracker_directly() {
    // `request_context` emits `client_ip`, which is the field `ip_changed` reads, so
    // a session created from a context needs no renaming.
    let out = stdout_of(&format!(
        r#"{REALISTIC_REQUEST}
fn main() {{
    let session = request_context(request())?
    println(str(ip_changed(session, "203.0.113.7")))
    println(str(ip_changed(session, "198.51.100.22")))
}}"#
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "true"],
        "full output: {out}"
    );
}

#[test]
fn request_context_names_a_bad_argument() {
    let out = stdout_of(
        r#"
fn main() {
    println(request_context("not a request").err())

    let bad_headers = map()
    bad_headers["method"] = "GET"
    bad_headers["path"] = "/"
    bad_headers["query"] = ""
    bad_headers["headers"] = "oops"
    println(request_context(bad_headers).err())

    let missing = map()
    missing["method"] = "GET"
    println(request_context(missing).err())
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert!(
        lines[0].contains("request_context: request") && lines[0].contains("str"),
        "got: {}",
        lines[0]
    );
    assert!(
        lines[1].contains("headers"),
        "the mistyped field must be named, got: {}",
        lines[1]
    );
    assert!(
        lines[2].contains("path"),
        "the missing field must be named, got: {}",
        lines[2]
    );
}

#[test]
fn identity_from_claims_names_a_bad_subject_type() {
    // An int user id is not coerced: `1` and `"1"` must not be the same principal
    // in one code path and different in another.
    let out = stdout_of(
        r#"
fn main() {
    let claims = map()
    claims["sub"] = 4711
    let attempt = identity_from_claims(claims)
    println(str(attempt.is_err()))
    println(attempt.err())
    println(identity_from_claims(7).err())
}"#,
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true");
    assert!(lines[1].contains("`sub` must be str"), "got: {}", lines[1]);
    assert!(
        lines[2].contains("claims must be a map"),
        "got: {}",
        lines[2]
    );
}
