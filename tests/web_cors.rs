//! Integration coverage for the CORS built-ins.
//!
//! CORS is a deliberate same-origin-policy bypass, so the load-bearing cases here
//! are the *refusals*: wildcard-plus-credentials rejected at construction, a
//! disallowed origin producing no header at all, and a preflight asking for a
//! method the policy never allowed. Each case runs a real `.tether` program
//! through the binary, because the concern modules are private and the script
//! surface is what a handler actually consumes.
//!
//! This file follows the size convention of its sibling `tests/web_*.rs` suites
//! (238–266 effective lines each): the 50-line rule is enforced on
//! `src/**/*.rs`, and splitting integration cases across files would hide which
//! built-in group they cover.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_cors_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("cors_case_{case}.tether"));
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
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .trim_end()
        .to_string()
}

/// A helper prelude: a credentialed policy for one exact origin, and a request
/// builder so each case only states what it varies.
const PRELUDE: &str = r#"
fn policy_for(credentials) {
    let config = map()
    config.origins = ["https://app.example.com"]
    config.methods = ["GET", "POST"]
    config.headers = ["content-type", "authorization"]
    config.expose = ["x-request-id", "x-total-count"]
    config.credentials = credentials
    config.max_age = 600
    return cors_policy(config)
}

fn request(method, headers) {
    let req = map()
    req.method = method
    req.path = "/api/things"
    req.query = ""
    req.headers = headers
    req.body = ""
    return req
}

fn origin_headers(origin) {
    let h = map()
    h["origin"] = origin
    return h
}
"#;

/// Compose the prelude with a case body.
fn program(body: &str) -> String {
    format!("{PRELUDE}\nfn main() {{{body}\n}}\n")
}

#[test]
fn policy_construction_accepts_a_good_config() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    println(str(p.credentials))
    println(str(p.wildcard))
    println(str(p.max_age))
    println(p.origins[0])
    println(p.methods[0] + "," + p.methods[1])
    println(p.headers[0])
    println(p.expose[0])"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        [
            "true",
            "false",
            "600",
            "https://app.example.com",
            "GET,POST",
            "content-type",
            "x-request-id",
        ],
        "full output: {out}"
    );
}

#[test]
fn wildcard_with_credentials_is_rejected_naming_the_conflict() {
    // The Fetch spec forbids the pair, and it is a credential leak: any origin on
    // the internet could read authenticated responses. Caught once, at startup.
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origins = "*"
    config.credentials = true
    let result = cors_policy(config)
    println(str(result.is_err()))
    println(result.err())"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "construction must fail: {out}");
    let message = lines[1..].join(" ");
    assert!(
        message.contains("origins") && message.contains("credentials"),
        "message must name both conflicting fields: {message}"
    );
    assert!(
        message.contains('*'),
        "message must name the wildcard: {message}"
    );
}

#[test]
fn wildcard_without_credentials_is_accepted() {
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origins = "*"
    let p = cors_policy(config)?
    println(str(p.wildcard))
    let req = request("GET", origin_headers("https://anywhere.example"))
    println(cors_headers(p, req)?["access-control-allow-origin"])"#,
    ));
    assert_eq!(out.lines().collect::<Vec<_>>(), ["true", "*"]);
}

#[test]
fn an_allowed_origin_is_echoed_exactly() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let req = request("GET", origin_headers("https://app.example.com"))
    let h = cors_headers(p, req)?
    println(h["access-control-allow-origin"])
    println(h["access-control-allow-credentials"])"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["https://app.example.com", "true"],
        "with credentials the exact origin is echoed, never `*`: {out}"
    );
}

#[test]
fn a_disallowed_origin_produces_no_allow_origin_header() {
    // Absent, not empty and not a wildcard fallback: an empty value is malformed
    // and a wildcard would defeat the allow-list.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let req = request("GET", origin_headers("https://evil.example.net"))
    let h = cors_headers(p, req)?
    println(str(h.len()))
    println(str(h.contains("access-control-allow-origin")))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["0", "false"],
        "a rejected origin gets no CORS headers at all: {out}"
    );
}

#[test]
fn origin_comparison_is_exact_across_scheme_host_and_port() {
    // Suffix or substring matching would admit every one of these. `evil-example`
    // ends with the allowed host's tail; `.evil.net` merely contains it.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(false)?
    let probes = [
        "http://app.example.com",
        "https://app.example.com:8443",
        "https://APP.example.com",
        "https://evil-app.example.com",
        "https://app.example.com.evil.net",
        "https://app.example.com/",
        "https://example.com"
    ]
    for probe in probes {
        let h = cors_headers(p, request("GET", origin_headers(probe)))?
        println(str(h.contains("access-control-allow-origin")))
    }
    let good = cors_headers(p, request("GET", origin_headers("https://app.example.com")))?
    println(str(good.contains("access-control-allow-origin")))"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 8, "full output: {out}");
    assert!(
        lines[..7].iter().all(|line| *line == "false"),
        "a different scheme, port, case, suffix, or path must not match: {out}"
    );
    assert_eq!(lines[7], "true", "the exact origin must match: {out}");
}

#[test]
fn vary_origin_is_present_whenever_the_origin_is_echoed() {
    // Without `Vary`, a shared cache keyed only on the URL serves origin A's
    // `Allow-Origin: A` to origin B.
    let out = stdout_of(&program(
        r#"
    let credentialed = policy_for(true)?
    let plain = policy_for(false)?
    let req = request("GET", origin_headers("https://app.example.com"))
    println(cors_headers(credentialed, req)?["vary"])
    println(cors_headers(plain, req)?["vary"])

    let pre_headers = origin_headers("https://app.example.com")
    pre_headers["access-control-request-method"] = "POST"
    let pre = cors_preflight(credentialed, request("OPTIONS", pre_headers))?
    println(pre.headers["vary"])"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["Origin", "Origin", "Origin"],
        "every echoed origin, credentialed or not, needs Vary: {out}"
    );
}

#[test]
fn a_wildcard_policy_without_credentials_need_not_vary() {
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origins = "*"
    let p = cors_policy(config)?
    let h = cors_headers(p, request("GET", origin_headers("https://any.example")))?
    println(str(h.contains("vary")))"#,
    ));
    assert_eq!(
        out, "false",
        "a bare `*` does not depend on the request, so nothing varies"
    );
}

#[test]
fn a_preflight_produces_204_with_allow_methods_and_allow_headers() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "POST"
    headers["access-control-request-headers"] = "content-type"
    let resp = cors_preflight(p, request("OPTIONS", headers))?
    println(str(resp.status))
    println(resp.headers["access-control-allow-methods"])
    println(resp.headers["access-control-allow-headers"])
    println(resp.headers["access-control-allow-origin"])
    println(resp.body)"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "204", "a preflight answer carries no body: {out}");
    assert_eq!(lines[1], "GET, POST");
    assert_eq!(lines[2], "content-type, authorization");
    assert_eq!(lines[3], "https://app.example.com");
    assert_eq!(lines.get(4).copied().unwrap_or(""), "");
}

#[test]
fn max_age_is_emitted_on_the_preflight_answer() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(false)?
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "GET"
    println(cors_preflight(p, request("OPTIONS", headers))?.headers["access-control-max-age"])

    let bare = map()
    bare.origins = ["https://app.example.com"]
    let no_age = cors_policy(bare)?
    let resp = cors_preflight(no_age, request("OPTIONS", headers))?
    println(str(resp.headers.contains("access-control-max-age")))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["600", "false"],
        "max_age is emitted when set and omitted when not: {out}"
    );
}

#[test]
fn a_non_preflight_options_request_is_not_treated_as_one() {
    // A bare OPTIONS is a legitimate capability probe. Answering it with a 204
    // would swallow the real handler.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(false)?
    let req = request("OPTIONS", origin_headers("https://app.example.com"))
    println(str(is_preflight(req)))
    println(str(cors_preflight(p, req)?))

    let get = request("GET", origin_headers("https://app.example.com"))
    println(str(is_preflight(get)))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["false", "nil", "false"],
        "only OPTIONS + Access-Control-Request-Method is a preflight: {out}"
    );
}

#[test]
fn is_preflight_requires_both_the_method_and_the_header() {
    let out = stdout_of(&program(
        r#"
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "POST"
    println(str(is_preflight(request("OPTIONS", headers))))
    // The header alone, on a non-OPTIONS method, is not a preflight.
    println(str(is_preflight(request("POST", headers))))"#,
    ));
    assert_eq!(out.lines().collect::<Vec<_>>(), ["true", "false"]);
}

#[test]
fn a_preflight_requesting_a_disallowed_method_is_refused() {
    // Refused, never reflected: reflecting the requested method would turn the
    // allow-list into an echo chamber.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "DELETE"
    let result = cors_preflight(p, request("OPTIONS", headers))
    println(str(result.is_err()))
    println(result.err())"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "DELETE is not on the list: {out}");
    let message = lines[1..].join(" ");
    assert!(
        message.contains("DELETE"),
        "the error must name the refused method: {message}"
    );
    assert!(
        message.contains("GET") && message.contains("POST"),
        "the error should say what is allowed: {message}"
    );
}

#[test]
fn a_preflight_requesting_a_disallowed_header_is_refused() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "POST"
    headers["access-control-request-headers"] = "content-type, x-admin-override"
    let result = cors_preflight(p, request("OPTIONS", headers))
    println(str(result.is_err()))
    println(result.err())"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1..].join(" ").contains("x-admin-override"),
        "the error must name the refused header: {out}"
    );
}

#[test]
fn a_preflight_from_a_disallowed_origin_is_refused() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let headers = origin_headers("https://evil.example.net")
    headers["access-control-request-method"] = "GET"
    println(cors_preflight(p, request("OPTIONS", headers)).err())"#,
    ));
    assert!(
        out.contains("https://evil.example.net"),
        "the error must name the origin it refused: {out}"
    );
}

#[test]
fn expose_headers_appear_on_the_actual_response_only() {
    // `Access-Control-Expose-Headers` is the only way script reads a
    // non-safelisted response header, so it belongs on the real response.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(false)?
    let req = request("GET", origin_headers("https://app.example.com"))
    println(cors_headers(p, req)?["access-control-expose-headers"])

    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "GET"
    let pre = cors_preflight(p, request("OPTIONS", headers))?
    println(str(pre.headers.contains("access-control-expose-headers")))

    let actual = cors_headers(p, req)?
    println(str(actual.contains("access-control-allow-methods")))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["x-request-id, x-total-count", "false", "false"],
        "expose belongs to the response, allow-methods to the preflight: {out}"
    );
}

#[test]
fn request_header_names_match_case_insensitively() {
    // HTTP header names are case-insensitive (RFC 9110 §5.1), and a hand-built
    // map may not be normalized. The *origin value*, by contrast, is exact.
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    let headers = map()
    headers["Origin"] = "https://app.example.com"
    headers["Access-Control-Request-Method"] = "post"
    headers["ACCESS-CONTROL-REQUEST-HEADERS"] = "Content-Type, AUTHORIZATION"
    let resp = cors_preflight(p, request("options", headers))?
    println(str(resp.status))
    println(resp.headers["access-control-allow-origin"])

    // Same map, but the origin's own case is changed: that must not match.
    let shouted = map()
    shouted["ORIGIN"] = "https://APP.example.com"
    shouted["access-control-request-method"] = "POST"
    println(str(cors_preflight(p, request("OPTIONS", shouted)).is_err()))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["204", "https://app.example.com", "true"],
        "names fold case, origin values do not: {out}"
    );
}

#[test]
fn an_unknown_config_key_is_rejected() {
    // `origin` instead of `origins` would otherwise build a policy that allows
    // nothing, and the symptom points nowhere near the typo.
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origin = ["https://app.example.com"]
    println(cors_policy(config).err())"#,
    ));
    assert!(
        out.contains("origin"),
        "the error must name the unknown key: {out}"
    );
}

#[test]
fn a_malformed_origin_is_rejected_at_construction() {
    let out = stdout_of(&program(
        r#"
    let cases = [
        ["app.example.com"],
        ["https://app.example.com/"],
        ["https://app.example.com/api"],
        ["*"],
        []
    ]
    for entry in cases {
        let config = map()
        config.origins = entry
        println(str(cors_policy(config).is_err()))
    }"#,
    ));
    assert!(
        out.lines().all(|line| line == "true"),
        "a missing scheme, a path, a trailing slash, a list wildcard, and an \
         empty list must all fail: {out}"
    );
}

#[test]
fn a_negative_max_age_is_rejected() {
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origins = ["https://app.example.com"]
    config.max_age = -1
    println(cors_policy(config).err())"#,
    ));
    assert!(
        out.contains("max_age"),
        "the error must name the field: {out}"
    );
}

#[test]
fn omitted_methods_default_to_read_only() {
    let out = stdout_of(&program(
        r#"
    let config = map()
    config.origins = ["https://app.example.com"]
    let p = cors_policy(config)?
    let headers = origin_headers("https://app.example.com")
    headers["access-control-request-method"] = "POST"
    println(str(cors_preflight(p, request("OPTIONS", headers)).is_err()))
    headers["access-control-request-method"] = "GET"
    println(str(cors_preflight(p, request("OPTIONS", headers))?.status))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "204"],
        "an omitted `methods` must not enable writes: {out}"
    );
}

#[test]
fn a_request_without_an_origin_gets_no_cors_headers() {
    let out = stdout_of(&program(
        r#"
    let p = policy_for(true)?
    println(str(cors_headers(p, request("GET", map()))?.len()))"#,
    ));
    assert_eq!(out, "0", "a same-origin request needs no CORS headers");
}

#[test]
fn the_null_origin_is_never_allow_listed() {
    // `Origin: null` is sent by sandboxed iframes, `file://` documents, and some
    // redirects, so it identifies no one: any attacker can present it by opening
    // their own page in a sandboxed frame. It must not be allow-listable, and it
    // must not match an exact-origin policy.
    let out = stdout_of(&program(
        r#"
    // `null` carries no scheme, so it cannot even be written into an allow-list.
    let attempted = map()
    attempted.origins = ["null"]
    println(str(cors_policy(attempted).is_err()))

    // And it is not echoed by a real policy.
    let p = policy_for(true)?
    let h = cors_headers(p, request("GET", origin_headers("null")))?
    println(str(h.contains("access-control-allow-origin")))

    let headers = origin_headers("null")
    headers["access-control-request-method"] = "POST"
    println(str(cors_preflight(p, request("OPTIONS", headers)).is_err()))"#,
    ));
    assert_eq!(
        out.lines().collect::<Vec<_>>(),
        ["true", "false", "true"],
        "`null` is a shared bucket, never an identity: {out}"
    );
}

#[test]
fn malformed_arguments_are_named_in_the_error() {
    let out = stdout_of(&program(
        r#"
    println(cors_policy(42).err())
    let p = policy_for(false)?
    println(cors_headers(p, "not a map").err())
    println(is_preflight(map()))"#,
    ));
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("cors_policy"), "got: {}", lines[0]);
    assert!(lines[1].contains("cors_headers"), "got: {}", lines[1]);
    assert_eq!(lines[2], "false", "an empty map is simply not a preflight");
}
