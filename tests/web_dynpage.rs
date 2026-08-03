//! Integration coverage for the dynamic-page built-ins.
//!
//! Two properties carry almost all of the weight here.
//!
//! * **A slug is rejected, never repaired.** A traversal, a percent-encoded
//!   traversal, a backslash, a NUL, and an over-long value must all be `Err`. A
//!   sanitising implementation would turn `..%2f..%2fetc` into `etc` and serve a
//!   real page nobody asked for, so each shape is asserted separately.
//! * **The cache key separates every varying input.** Each input is varied on its
//!   own and the key must change, and a component-boundary collision attempt — the
//!   kind a printable separator such as `-` would let through — must not collide.
//!   Omitting `authenticated` would serve a signed-in visitor's page to an
//!   anonymous one, so that case is asserted twice: it must move the key *and*
//!   mark it `private`.
//!
//! Every case runs a real `.tether` program through the binary, so these exercise
//! the registered built-ins rather than the private Rust functions behind them.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its trimmed stdout, asserting the run succeeded.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_dynpage_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::write(&path, source).expect("source should be writable");
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
        .trim()
        .to_string()
}

/// Split captured stdout for per-line assertions.
fn lines(out: &str) -> Vec<&str> {
    out.lines().collect()
}

/// Helpers every case reuses: a one-header request map, and a parts map.
const PRELUDE: &str = r#"
fn req(name, value) {
    let h = map()
    if name != "" {
        h[name] = value
    }
    let r = map()
    r.path = "/p"
    r.headers = h
    return r
}

fn parts(slug, locale, variant, device, auth) {
    let p = map()
    p.slug = slug
    p.locale = locale
    p.variant = variant
    p.device = device
    p.authenticated = auth
    return p
}

fn base() {
    return parts("about", "en", "control", "desktop", false)
}
"#;

/// Run a case with the shared prelude prepended.
fn run_with(source: &str) -> String {
    run(&format!("{PRELUDE}\n{source}"))
}

// ------------------------------------------------------------------ slug parse

#[test]
fn slug_is_parsed_from_several_path_shapes() {
    let out = run(r#"fn main() {
    println(slug_parse("/about")?)
    println(slug_parse("about")?)
    println(slug_parse("about/")?)
    println(slug_parse("/about/")?)
    println(slug_parse("/About/")?)
    println(slug_parse("my-page_2")?)
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "about", "a leading slash must be stripped: {out}");
    assert_eq!(got[1], "about", "a bare slug must pass through: {out}");
    assert_eq!(got[2], "about", "a trailing slash must be stripped: {out}");
    assert_eq!(got[3], "about", "both slashes must be stripped: {out}");
    assert_eq!(got[4], "about", "the slug must be lowercased: {out}");
    assert_eq!(
        got[5], "my-page_2",
        "`-`, `_`, and digits are in the charset: {out}"
    );
}

#[test]
fn an_empty_or_slash_only_path_is_rejected() {
    let out = run(r#"fn main() {
    println(str(slug_parse("").is_err()))
    println(str(slug_parse("/").is_err()))
    println(slug_parse("///").err())
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "an empty path has no slug: {out}");
    assert_eq!(got[1], "true", "`/` normalises to empty: {out}");
    assert!(
        got[2].contains("empty"),
        "the error must say the slug was empty, got: {}",
        got[2]
    );
}

// ------------------------------------------------------------- slug rejections

/// Rejecting beats sanitising: stripping `..` from `..%2f..%2fetc` would yield
/// `etc`, a real page the attacker never requested.
#[test]
fn a_traversal_attempt_is_rejected() {
    let out = run(r#"fn main() {
    let dots = slug_parse("/../")
    let nested = slug_parse("/../etc/passwd")
    println(str(dots.is_err()))
    println(dots.err())
    println(str(nested.is_err()))
    println(nested.err())
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "`..` must not be accepted: {out}");
    assert!(
        got[1].contains("traversal"),
        "the error must name the traversal, got: {}",
        got[1]
    );
    assert_eq!(
        got[2], "true",
        "a nested traversal must not be accepted: {out}"
    );
    assert!(
        got[3].contains("separator") || got[3].contains("traversal"),
        "the error must name the offending construct, got: {}",
        got[3]
    );
}

/// The encoded form must fail too. Decoding here would undo the segment-then-decode
/// ordering `route_decode.rs` relies on, letting `%2F` become a real separator.
#[test]
fn a_percent_encoded_traversal_is_rejected() {
    let out = run(r#"fn main() {
    let a = slug_parse("%2e%2e%2fetc")
    let b = slug_parse("..%2fetc")
    let c = slug_parse("%2E%2E")
    println(str(a.is_err()))
    println(a.err())
    println(str(b.is_err()))
    println(str(c.is_err()))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "`%2e%2e%2f` must not be accepted: {out}");
    assert!(
        got[1].contains("percent"),
        "the error must name the percent escape, got: {}",
        got[1]
    );
    assert_eq!(got[2], "true", "a mixed encoded traversal must fail: {out}");
    assert_eq!(got[3], "true", "upper-case hex must fail too: {out}");
}

#[test]
fn a_backslash_is_rejected() {
    let out = run(r#"fn main() {
    let bad = slug_parse("a\\b")
    println(str(bad.is_err()))
    println(bad.err())
    println(str(slug_parse("..\\..\\win").is_err()))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a backslash must not be accepted: {out}");
    assert!(
        got[1].contains("backslash"),
        "the error must name the backslash, got: {}",
        got[1]
    );
    assert_eq!(got[2], "true", "a Windows-style traversal must fail: {out}");
}

/// A NUL truncates a C string, so a slug containing one could name a different file
/// than the one validated. It is built through `bytes` because the lexer has no
/// `\0` escape.
#[test]
fn a_nul_byte_is_rejected() {
    let out = run(r#"fn main() {
    let evil = bytes([97, 0, 98]).decode_utf8()
    let bad = slug_parse(evil)
    println(str(bad.is_err()))
    println(bad.err())
    println(str(slug_valid(evil)))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a NUL must not be accepted: {out}");
    assert!(
        got[1].contains("NUL"),
        "the error must name the NUL byte, got: {}",
        got[1]
    );
    assert_eq!(got[2], "false", "slug_valid must agree: {out}");
}

/// The limit is 200 bytes, so a filesystem component built from the slug still fits
/// inside the usual 255-byte cap with room for a prefix and an extension.
#[test]
fn an_over_long_slug_is_rejected_and_the_limit_is_stated() {
    let out = run(r#"fn main() {
    let mut long = ""
    let mut i = 0
    while i < 201 {
        long = long + "a"
        i = i + 1
    }
    let bad = slug_parse(long)
    println(str(bad.is_err()))
    println(bad.err())
    let mut edge = ""
    let mut j = 0
    while j < 200 {
        edge = edge + "a"
        j = j + 1
    }
    println(str(slug_parse(edge).is_err()))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "201 bytes must be refused: {out}");
    assert!(
        got[1].contains("200") && got[1].contains("201"),
        "the error must state the limit and the length seen, got: {}",
        got[1]
    );
    assert_eq!(got[2], "false", "exactly 200 bytes must be accepted: {out}");
}

#[test]
fn slug_valid_reports_the_charset_without_normalising() {
    let out = run(r#"fn main() {
    println(str(slug_valid("about")))
    println(str(slug_valid("my-page_2")))
    println(str(slug_valid("About")))
    println(str(slug_valid("a/b")))
    println(str(slug_valid("a b")))
    println(str(slug_valid("a.b")))
    println(str(slug_valid("")))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a plain slug is valid: {out}");
    assert_eq!(got[1], "true", "`-` and `_` are in the charset: {out}");
    assert_eq!(
        got[2], "false",
        "slug_valid answers `usable as-is`, so uppercase is false: {out}"
    );
    assert_eq!(got[3], "false", "a separator is never valid: {out}");
    assert_eq!(got[4], "false", "a space is not in the charset: {out}");
    assert_eq!(got[5], "false", "a dot is not in the charset: {out}");
    assert_eq!(got[6], "false", "an empty slug is not valid: {out}");
}

// ------------------------------------------------------------------ cache keys

/// Every input the render varies on must move the key. Omitting any one of these is
/// how two different renders collapse onto a single cache entry.
#[test]
fn the_cache_key_changes_for_every_varying_input_in_turn() {
    let out = run_with(r#"fn main() {
    let k = page_cache_key(base())?
    println(str(k == page_cache_key(parts("other", "en", "control", "desktop", false))?))
    println(str(k == page_cache_key(parts("about", "es", "control", "desktop", false))?))
    println(str(k == page_cache_key(parts("about", "en", "treatment", "desktop", false))?))
    println(str(k == page_cache_key(parts("about", "en", "control", "mobile", false))?))
    println(str(k == page_cache_key(parts("about", "en", "control", "desktop", true))?))
    println(str(k == page_cache_key(base())?))
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "false",
        "a different slug must not share a key: {out}"
    );
    assert_eq!(
        got[1], "false",
        "omitting locale would serve the wrong language: {out}"
    );
    assert_eq!(got[2], "false", "a variant must move the key: {out}");
    assert_eq!(got[3], "false", "a device class must move the key: {out}");
    assert_eq!(
        got[4], "false",
        "omitting the authenticated flag would serve a signed-in page to an anonymous visitor: {out}"
    );
    assert_eq!(
        got[5], "true",
        "the key must be stable for equal inputs: {out}"
    );
}

/// `-` is legal inside a slug, so a printable separator would let `a-b` + `c` and
/// `a` + `b-c` collide. The `0x1F` unit separator cannot appear in any component,
/// which is what makes the join injective.
#[test]
fn a_component_boundary_collision_attempt_does_not_collide() {
    let out = run_with(r#"fn main() {
    println(str(page_cache_key(parts("a-b", "c", "", "", false))? == page_cache_key(parts("a", "b-c", "", "", false))?))
    println(str(page_cache_key(parts("a", "", "b", "", false))? == page_cache_key(parts("a", "b", "", "", false))?))
    println(str(page_cache_key(parts("ab", "c", "", "", false))? == page_cache_key(parts("a", "bc", "", "", false))?))
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "false",
        "`a-b`+`c` must not collide with `a`+`b-c`: {out}"
    );
    assert_eq!(
        got[1], "false",
        "an empty component must not let a value slide into the next slot: {out}"
    );
    assert_eq!(
        got[2], "false",
        "`ab`+`c` must not collide with `a`+`bc`: {out}"
    );
}

/// A shared cache must be able to exclude a private render with one prefix test.
#[test]
fn an_authenticated_key_is_marked_private_and_anonymous_public() {
    let out = run_with(r#"fn main() {
    let private = page_cache_key(parts("about", "en", "", "", true))?
    let public = page_cache_key(parts("about", "en", "", "", false))?
    println(str(private.starts_with("private")))
    println(str(public.starts_with("public")))
    println(str(private.contains("about")))
    println(str(private == public))
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "true",
        "an authenticated key must be excludable by prefix: {out}"
    );
    assert_eq!(
        got[1], "true",
        "an anonymous key must be marked public: {out}"
    );
    assert_eq!(
        got[2], "true",
        "a private key still carries its inputs so a per-session cache works: {out}"
    );
    assert_eq!(
        got[3], "false",
        "a private and a public render must never share an entry: {out}"
    );
}

#[test]
fn optional_key_inputs_may_be_omitted_entirely() {
    let out = run_with(r#"fn main() {
    let bare = map()
    bare.slug = "about"
    println(str(page_cache_key(bare).is_err()))
    println(str(page_cache_key(bare)? == page_cache_key(parts("about", "", "", "", false))?))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "false", "only `slug` is required: {out}");
    assert_eq!(
        got[1], "true",
        "an absent optional input must equal an empty one: {out}"
    );
}

/// A missing `authenticated` defaults to false — the *cacheable* answer — so the
/// error paths below are the only way a bad flag can reach a key.
#[test]
fn a_malformed_parts_map_is_rejected_naming_the_field() {
    let out = run_with(r#"fn main() {
    let no_slug = map()
    no_slug.locale = "en"
    println(page_cache_key(no_slug).err())
    let bad_slug = map()
    bad_slug.slug = "../etc"
    println(str(page_cache_key(bad_slug).is_err()))
    let bad_flag = map()
    bad_flag.slug = "about"
    bad_flag.authenticated = "yes"
    println(page_cache_key(bad_flag).err())
    println(page_cache_key("not a map").err())
    let bad_locale = map()
    bad_locale.slug = "about"
    bad_locale.locale = "en us"
    println(page_cache_key(bad_locale).err())
}"#);
    let got = lines(&out);
    assert!(
        got[0].contains("slug"),
        "a missing slug must be named, got: {}",
        got[0]
    );
    assert_eq!(got[1], "true", "an unsafe slug must not reach a key: {out}");
    assert!(
        got[2].contains("authenticated") && got[2].contains("bool"),
        "a wrong flag type must name the field and the type, got: {}",
        got[2]
    );
    assert!(
        got[3].contains("must be a map") && got[3].contains("str"),
        "a non-map must name the expected and the actual type, got: {}",
        got[3]
    );
    assert!(
        got[4].contains("locale"),
        "a locale outside the charset must be named, got: {}",
        got[4]
    );
}

// ------------------------------------------------------------------------ vary

/// Too few entries poisons the cache; too many defeats it. So the list must match
/// the consumed inputs exactly.
#[test]
fn vary_headers_lists_exactly_the_inputs_the_key_consumed() {
    let out = run_with(r#"fn main() {
    println("[" + vary_headers(parts("about", "", "", "", false))? + "]")
    println(vary_headers(parts("about", "en", "", "", false))?)
    println(vary_headers(parts("about", "", "", "mobile", false))?)
    println(vary_headers(parts("about", "", "", "", true))?)
    println(vary_headers(base())?)
    println(vary_headers(parts("about", "en", "control", "desktop", true))?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "[]",
        "a slug comes from the path, so no header is consumed: {out}"
    );
    assert_eq!(
        got[1], "Accept-Language",
        "a locale consumes exactly Accept-Language: {out}"
    );
    assert_eq!(
        got[2], "User-Agent",
        "a device class consumes exactly User-Agent: {out}"
    );
    assert_eq!(
        got[3], "Cookie, Authorization",
        "either header can carry the credential that made the render private: {out}"
    );
    assert_eq!(
        got[4], "Accept-Language, User-Agent",
        "a variant alone adds no header: {out}"
    );
    assert_eq!(
        got[5], "Accept-Language, User-Agent, Cookie, Authorization",
        "every consumed header must be listed: {out}"
    );
}

/// `Vary: User-Agent` is effectively unique per client, so listing it when the key
/// ignores the device would collapse the hit rate to nothing.
#[test]
fn vary_headers_omits_user_agent_when_the_key_ignores_the_device() {
    let out = run_with(r#"fn main() {
    let v = vary_headers(parts("about", "en", "control", "", false))?
    println(str(v.contains("User-Agent")))
    println(v)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "false",
        "listing User-Agent unconditionally defeats the cache: {out}"
    );
    assert_eq!(got[1], "Accept-Language", "the locale is still listed: {out}");
}

#[test]
fn vary_headers_rejects_the_same_bad_parts_as_the_key() {
    let out = run_with(r#"fn main() {
    let bad = map()
    bad.slug = "a/b"
    println(str(vary_headers(bad).is_err()))
    println(str(vary_headers(42).is_err()))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "the two must agree about validity: {out}");
    assert_eq!(got[1], "true", "a non-map must be refused: {out}");
}

// ----------------------------------------------------------- conditional replies

#[test]
fn a_matching_validator_yields_a_304_and_a_mismatch_yields_nil() {
    let out = run_with(r#"fn main() {
    let tag = etag_of("<h1>hi</h1>")
    let hit = page_not_modified(tag, req("if-none-match", tag))?
    println(str(hit != nil))
    println(str(hit.status))
    println(str(hit.body == ""))
    println(str(hit.headers["etag"] == tag))
    let miss = page_not_modified(tag, req("if-none-match", etag_of("other")))?
    println(str(miss == nil))
    let absent = page_not_modified(tag, req("", ""))?
    println(str(absent == nil))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a matching validator must answer 304: {out}");
    assert_eq!(got[1], "304", "the status must be 304: {out}");
    assert_eq!(got[2], "true", "RFC 9110 forbids a body on 304: {out}");
    assert_eq!(
        got[3], "true",
        "RFC 9110 requires the validator on a 304: {out}"
    );
    assert_eq!(
        got[4], "true",
        "a mismatch must be nil rather than an error: {out}"
    );
    assert_eq!(
        got[5], "true",
        "an absent If-None-Match must be nil: {out}"
    );
}

/// A prefix hit would answer 304 for a body the client has never seen, which is
/// worse than not caching at all.
#[test]
fn validator_comparison_is_exact_and_handles_weak_tags_and_lists() {
    let out = run_with(r#"fn main() {
    println(str(page_not_modified("\"abc\"", req("if-none-match", "\"abcdef\""))? == nil))
    println(str(page_not_modified("\"abc\"", req("if-none-match", "W/\"abc\""))? != nil))
    println(str(page_not_modified("\"abc\"", req("if-none-match", "\"x\", \"abc\""))? != nil))
    println(str(page_not_modified("\"abc\"", req("if-none-match", "*"))? != nil))
    println(str(page_not_modified("", req("if-none-match", "*"))? != nil))
    println(str(page_not_modified("\"abc\"", req("If-None-Match", "\"abc\""))? != nil))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a prefix must not match: {out}");
    assert_eq!(
        got[1], "true",
        "weak comparison must match a strong tag: {out}"
    );
    assert_eq!(got[2], "true", "a list entry must match: {out}");
    assert_eq!(got[3], "true", "`*` must match any representation: {out}");
    assert_eq!(
        got[4], "false",
        "an empty cached tag vouches for nothing, so `*` must not match: {out}"
    );
    assert_eq!(got[5], "true", "header lookup must ignore case: {out}");
}

#[test]
fn page_not_modified_rejects_a_bad_argument_shape() {
    let out = run_with(r#"fn main() {
    println(page_not_modified(7, req("", "")).err())
    println(str(page_not_modified("\"a\"", "not a map").is_err()))
    let bad = map()
    bad.headers = "not a map"
    println(str(page_not_modified("\"a\"", bad).is_err()))
}"#);
    let got = lines(&out);
    assert!(
        got[0].contains("cached_etag") && got[0].contains("int"),
        "the error must name the parameter and the actual type, got: {}",
        got[0]
    );
    assert_eq!(got[1], "true", "a non-map request must be refused: {out}");
    assert_eq!(
        got[2], "true",
        "a non-map headers field must be refused: {out}"
    );
}

// ---------------------------------------------------------------- device class

#[test]
fn device_class_recognises_a_phone_a_tablet_and_a_desktop() {
    let out = run_with(r#"fn main() {
    let phone = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148"
    let android = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36"
    let tablet = "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Mobile/15E148"
    let android_tab = "Mozilla/5.0 (Linux; Android 13; SM-X700) AppleWebKit/537.36 Chrome/120 Safari/537.36 Tablet"
    let desktop = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/120 Safari/537.36"
    println(device_class(req("user-agent", phone)))
    println(device_class(req("user-agent", android)))
    println(device_class(req("user-agent", tablet)))
    println(device_class(req("user-agent", android_tab)))
    println(device_class(req("user-agent", desktop)))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "mobile", "an iPhone is mobile: {out}");
    assert_eq!(got[1], "mobile", "an Android phone is mobile: {out}");
    assert_eq!(
        got[2], "tablet",
        "an iPad also says Mobile, so tablet must be tested first: {out}"
    );
    assert_eq!(
        got[3], "tablet",
        "an Android tablet must not be classed as a phone: {out}"
    );
    assert_eq!(got[4], "desktop", "a Mac browser is desktop: {out}");
}

#[test]
fn device_class_defaults_to_desktop_and_ignores_case() {
    let out = run_with(r#"fn main() {
    println(device_class(req("", "")))
    println(device_class(req("user-agent", "")))
    println(device_class(req("User-Agent", "iPhone")))
    println(device_class(req("user-agent", "curl/8.4.0")))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "desktop", "an absent User-Agent is desktop: {out}");
    assert_eq!(got[1], "desktop", "an empty User-Agent is desktop: {out}");
    assert_eq!(got[2], "mobile", "header lookup must ignore case: {out}");
    assert_eq!(got[3], "desktop", "an unknown agent is desktop: {out}");
}

// ---------------------------------------------------------- locale negotiation

#[test]
fn locale_negotiation_honours_q_value_ordering() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("accept-language", "en;q=0.5, es;q=0.9"), ["en", "es"])?)
    println(locale_of(req("accept-language", "es;q=0.1, en;q=0.8"), ["en", "es"])?)
    println(locale_of(req("accept-language", "es, en;q=0.9"), ["en", "es"])?)
    println(locale_of(req("accept-language", "de;q=1.0, es;q=0.9"), ["en", "es"])?)
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "es", "the higher q must win: {out}");
    assert_eq!(
        got[1], "en",
        "the higher q must win regardless of written order: {out}"
    );
    assert_eq!(got[2], "es", "a missing q defaults to 1: {out}");
    assert_eq!(
        got[3], "es",
        "an unsupported top choice must not block the next one: {out}"
    );
}

#[test]
fn an_unsupported_language_falls_back_to_the_default() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("accept-language", "de-DE, fr;q=0.8"), ["en", "es"])?)
    println(locale_of(req("accept-language", "zz"), ["es", "en"])?)
    println(locale_of(req("accept-language", ""), ["en", "es"])?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "en",
        "nothing matched, so the first supported wins: {out}"
    );
    assert_eq!(
        got[1], "es",
        "the caller controls the default by ordering its list: {out}"
    );
    assert_eq!(got[2], "en", "an empty header falls back: {out}");
}

#[test]
fn an_absent_accept_language_header_uses_the_default() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("", ""), ["en", "es"])?)
    println(locale_of(req("user-agent", "curl"), ["fr", "en"])?)
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "en", "an absent header is the default: {out}");
    assert_eq!(got[1], "fr", "the default is the caller's first entry: {out}");
}

#[test]
fn locale_negotiation_matches_a_dialect_by_primary_subtag() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("accept-language", "en-GB"), ["en", "es"])?)
    println(locale_of(req("accept-language", "en"), ["es", "en-us"])?)
    println(locale_of(req("accept-language", "en-GB"), ["en-us", "en-gb"])?)
    println(locale_of(req("accept-language", "*"), ["es", "en"])?)
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "en", "`en-GB` must accept supported `en`: {out}");
    assert_eq!(got[1], "en-us", "`en` must accept supported `en-us`: {out}");
    assert_eq!(
        got[2], "en-gb",
        "an exact match must beat a prefix match: {out}"
    );
    assert_eq!(got[3], "es", "`*` selects the first supported: {out}");
}

/// `Accept-Language` is attacker-controlled, so only the first 16 entries are
/// parsed. A match hiding past the bound is discarded, not searched for.
#[test]
fn more_entries_than_the_bound_are_discarded_rather_than_parsed() {
    let out = run_with(r#"fn main() {
    let mut header = ""
    let mut i = 0
    while i < 20 {
        if i > 0 {
            header = header + ","
        }
        header = header + "zz" + str(i) + ";q=0.9"
        i = i + 1
    }
    println(locale_of(req("accept-language", header + ",es;q=1.0"), ["en", "es"])?)
    println(locale_of(req("accept-language", "es;q=1.0," + header), ["en", "es"])?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "en",
        "entry 21 is past the 16-entry bound, so the default must be used: {out}"
    );
    assert_eq!(
        got[1], "es",
        "a match inside the bound must still be found: {out}"
    );
}

/// A garbage q must not be promoted to the RFC default of 1, or a malformed entry
/// would outrank a well-formed one.
#[test]
fn locale_negotiation_drops_a_malformed_q_rather_than_promoting_it() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("accept-language", "es;q=bogus, en;q=0.4"), ["en", "es"])?)
    println(locale_of(req("accept-language", "es;q=0, en"), ["en", "es"])?)
    println(locale_of(req("accept-language", "es;q=7, en;q=0.2"), ["en", "es"])?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "en",
        "a non-numeric q must not outrank a well-formed entry: {out}"
    );
    assert_eq!(got[1], "en", "`q=0` means not acceptable: {out}");
    assert_eq!(got[2], "en", "an out-of-range q must be dropped: {out}");
}

#[test]
fn locale_of_requires_a_non_empty_supported_list() {
    let out = run_with(r#"fn main() {
    println(locale_of(req("", ""), []).err())
    println(str(locale_of(req("", ""), "en").is_err()))
    println(str(locale_of(req("", ""), [1]).is_err()))
    println(str(locale_of("not a map", ["en"]).is_err()))
}"#);
    let got = lines(&out);
    assert!(
        got[0].contains("empty"),
        "an empty list must say so rather than inventing a locale, got: {}",
        got[0]
    );
    assert_eq!(got[1], "true", "a non-list must be refused: {out}");
    assert_eq!(got[2], "true", "a non-str entry must be refused: {out}");
    assert_eq!(got[3], "true", "a non-map request must be refused: {out}");
}

/// The negotiated value is always an element of `supported`, which is what keeps an
/// attacker-controlled header out of the cache key.
#[test]
fn a_hostile_accept_language_is_never_echoed_into_the_result() {
    let out = run_with(r#"fn main() {
    let hostile = "<script>alert(1)</script>;q=1.0, en;q=0.1"
    let got = locale_of(req("accept-language", hostile), ["en", "es"])?
    println(got)
    println(str(got.contains("script")))
    println(str(page_cache_key(parts("about", got, "", "", false))?.contains("script")))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "en", "the result must be a declared locale: {out}");
    assert_eq!(
        got[1], "false",
        "nothing from the header may be echoed: {out}"
    );
    assert_eq!(got[2], "false", "nothing hostile may reach the key: {out}");
}

// ------------------------------------------------------ end-to-end composition

/// The whole point: `route_match` supplies the segment, `slug_parse` validates it,
/// and the key, the `Vary` value, and the conditional reply all come from one shape.
#[test]
fn the_group_composes_with_route_matching_and_etags() {
    let out = run_with(r#"fn main() {
    let m = route_match("/blog/\{slug\}", "/blog/My-Post")?
    let p = map()
    p.slug = slug_parse(m.slug)?
    p.locale = locale_of(req("accept-language", "es"), ["en", "es"])?
    p.device = device_class(req("user-agent", "iPhone"))
    p.authenticated = false
    println(p.slug)
    println(p.locale)
    println(p.device)
    println(str(page_cache_key(p)?.starts_with("public")))
    println(vary_headers(p)?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "my-post",
        "the route capture must feed slug_parse: {out}"
    );
    assert_eq!(got[1], "es", "negotiation must run: {out}");
    assert_eq!(got[2], "mobile", "classification must run: {out}");
    assert_eq!(got[3], "true", "an anonymous render is shareable: {out}");
    assert_eq!(
        got[4], "Accept-Language, User-Agent",
        "Vary must match the consumed inputs: {out}"
    );
}
