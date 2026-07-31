//! Integration coverage for the date/time built-ins.
//!
//! Every expected value here is independently checkable rather than derived from
//! the implementation:
//!
//! * `0` is the epoch itself, which was a Thursday.
//! * `1445412480` is the `Wed, 21 Oct 2015 07:28:00 GMT` example from RFC 7231.
//! * `951782400` is 2020-02-29, a leap day that only exists under the correct
//!   Gregorian rule.
//! * 1900 and 2000 pin the century rule in both directions: 1900 is not a leap
//!   year, 2000 is.
//!
//! Round-trip assertions matter as much as the literals. Formatting and parsing
//! could share a compensating off-by-one and still round-trip, so the fixed
//! strings above are what catch that; the round trips catch the inverse.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its trimmed stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_datetime_{}", std::process::id()));
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
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn http_date_formats_the_epoch() {
    // 1970-01-01 was a Thursday. A hardcoded weekday table would get this wrong.
    let out = run(r#"
fn main() {
    println(http_date(0)?)
}
"#);
    assert_eq!(out, "Thu, 01 Jan 1970 00:00:00 GMT");
}

#[test]
fn http_date_formats_the_rfc7231_example() {
    let out = run(r#"
fn main() {
    println(http_date(1445412480)?)
}
"#);
    assert_eq!(out, "Wed, 21 Oct 2015 07:28:00 GMT");
}

#[test]
fn rfc3339_formats_the_same_instant() {
    let out = run(r#"
fn main() {
    println(rfc3339(0)?)
    println(rfc3339(1445412480)?)
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("1970-01-01T00:00:00Z"), "full: {out}");
    assert_eq!(lines.next(), Some("2015-10-21T07:28:00Z"), "full: {out}");
}

#[test]
fn leap_day_2020_02_29_formats_and_parses() {
    // 1582934400 is 2020-02-29T00:00:00Z, and 951782400 is 2000-02-29T00:00:00Z.
    // Both are leap days, and 2000 is the century that IS a leap year, so the two
    // together pin the divisible-by-400 branch. A wrong rule renders either as
    // March 1, one day out.
    let out = run(r#"
fn main() {
    println(rfc3339(1582934400)?)
    println(http_date(1582934400)?)
    println(str(rfc3339_parse("2020-02-29T00:00:00Z")?))
    println(rfc3339(951782400)?)
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("2020-02-29T00:00:00Z"), "full: {out}");
    assert_eq!(
        lines.next(),
        Some("Sat, 29 Feb 2020 00:00:00 GMT"),
        "full: {out}"
    );
    assert_eq!(lines.next(), Some("1582934400"), "full: {out}");
    assert_eq!(lines.next(), Some("2000-02-29T00:00:00Z"), "full: {out}");
}

#[test]
fn century_leap_rule_distinguishes_1900_from_2000() {
    // 2000 is divisible by 400 so it is a leap year; 1900 is a century that is
    // not, so 1900-02-29 does not exist and must be rejected.
    let out = run(r#"
fn main() {
    println(str(rfc3339_parse("2000-02-29T00:00:00Z").is_ok()))
    println(str(rfc3339_parse("1900-02-29T00:00:00Z").is_err()))
    println(rfc3339_parse("1900-02-29T00:00:00Z").err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "2000-02-29 must exist; full: {out}");
    assert_eq!(lines[1], "true", "1900-02-29 must not exist; full: {out}");
    assert!(
        lines[2].contains("day 29") && lines[2].contains("1-28"),
        "error should name the valid range, got: {}",
        lines[2]
    );
}

#[test]
fn pre_epoch_timestamps_format_correctly() {
    // -86400 is one day before the epoch: 1969-12-31, a Wednesday. Negative
    // timestamps are where naive truncating division goes wrong.
    let out = run(r#"
fn main() {
    println(rfc3339(-86400)?)
    println(http_date(-86400)?)
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("1969-12-31T00:00:00Z"), "full: {out}");
    assert_eq!(
        lines.next(),
        Some("Wed, 31 Dec 1969 00:00:00 GMT"),
        "full: {out}"
    );
}

#[test]
fn parse_round_trips_across_many_timestamps() {
    let out = run(r#"
fn main() {
    let stamps = [0, 1, 86399, 86400, 951782400, 1445412480, 2147483647, -1, -86400, -2208988800]
    let mut http_bad = 0
    let mut iso_bad = 0
    for stamp in stamps {
        if http_date_parse(http_date(stamp)?)? != stamp { http_bad = http_bad + 1 }
        if rfc3339_parse(rfc3339(stamp)?)? != stamp { iso_bad = iso_bad + 1 }
    }
    println(str(http_bad))
    println(str(iso_bad))
}
"#);
    let mut lines = out.lines();
    assert_eq!(
        lines.next(),
        Some("0"),
        "http_date round trip failed; {out}"
    );
    assert_eq!(lines.next(), Some("0"), "rfc3339 round trip failed; {out}");
}

#[test]
fn parses_the_rfc7231_example_back_to_seconds() {
    let out = run(r#"
fn main() {
    println(str(http_date_parse("Wed, 21 Oct 2015 07:28:00 GMT")?))
    println(str(http_date_parse("Thu, 01 Jan 1970 00:00:00 GMT")?))
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("1445412480"), "full: {out}");
    assert_eq!(lines.next(), Some("0"), "full: {out}");
}

#[test]
fn malformed_http_dates_report_the_specific_problem() {
    let out = run(r#"
fn main() {
    println(http_date_parse("not a date").err())
    println(http_date_parse("Wed, 21 Xxx 2015 07:28:00 GMT").err())
    println(http_date_parse("Wed, 21 Oct 2015 07:28:00 PST").err())
    println(http_date_parse("Wed, 32 Oct 2015 07:28:00 GMT").err())
    println(http_date_parse("Wed, 21 Oct 2015 25:00:00 GMT").err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("6 fields"), "got: {}", lines[0]);
    assert!(lines[1].contains("month abbreviation"), "got: {}", lines[1]);
    assert!(lines[2].contains("GMT"), "got: {}", lines[2]);
    assert!(lines[3].contains("day 32"), "got: {}", lines[3]);
    assert!(lines[4].contains("out of range"), "got: {}", lines[4]);
}

#[test]
fn malformed_rfc3339_values_report_the_specific_problem() {
    // A numeric offset is rejected rather than read as UTC: silently treating
    // +02:00 as Z would shift the instant by two hours.
    let out = run(r#"
fn main() {
    println(rfc3339_parse("2015-10-21").err())
    println(rfc3339_parse("2015-10-21T07:28:00+02:00").err())
    println(rfc3339_parse("2015-13-21T07:28:00Z").err())
    println(rfc3339_parse("2015-10-2aT07:28:00Z").err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("separator"), "got: {}", lines[0]);
    assert!(lines[1].contains("Z"), "got: {}", lines[1]);
    assert!(lines[2].contains("month 13"), "got: {}", lines[2]);
    assert!(lines[3].contains("not a decimal"), "got: {}", lines[3]);
}

#[test]
fn rfc3339_truncates_fractional_seconds() {
    let out = run(r#"
fn main() {
    println(str(rfc3339_parse("2015-10-21T07:28:00.512Z")?))
}
"#);
    assert_eq!(out, "1445412480", "fractional seconds should truncate");
}

#[test]
fn time_now_secs_agrees_with_time_now_ms() {
    // Not a fixed value: assert the relationship instead, allowing a second of
    // skew for a clock tick between the two calls.
    let out = run(r#"
fn main() {
    let secs = time_now_secs()
    let ms = time_now_ms()
    let delta = ms / 1000 - secs
    println(str(delta >= 0 && delta <= 1))
    println(str(secs > 1700000000))
}
"#);
    let mut lines = out.lines();
    assert_eq!(lines.next(), Some("true"), "seconds vs ms mismatch; {out}");
    assert_eq!(lines.next(), Some("true"), "clock looks unset; {out}");
}

#[test]
fn wrong_argument_types_are_named() {
    let out = run(r#"
fn main() {
    println(http_date("not an int").err())
    println(rfc3339_parse(42).err())
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].contains("must be int"), "got: {}", lines[0]);
    assert!(lines[1].contains("must be str"), "got: {}", lines[1]);
}

#[test]
fn expiry_one_week_out_is_a_valid_cookie_attribute() {
    // The motivating case: a session cookie needs a real Expires value.
    let out = run(r#"
fn main() {
    let expires = http_date(time_now_secs() + 604800)?
    let opts = map()
    opts.expires = expires
    opts.http_only = true
    let header = cookie_serialize("sid", "abc123", opts)?
    println(str(header.contains("Expires=")))
    println(str(header.contains("GMT")))
    println(str(http_date_parse(expires).is_ok()))
}
"#);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full: {out}");
    assert_eq!(lines[1], "true", "full: {out}");
    assert_eq!(lines[2], "true", "full: {out}");
}
