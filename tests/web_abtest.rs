//! Integration coverage for deterministic A/B test assignment.
//!
//! Two properties carry the most weight, and both are experiment-invalidating rather
//! than merely cosmetic.
//!
//! *Stability.* A visitor must get the same variant on every request, forever. If
//! assignment drifts, the experiment measures nothing and the site looks broken, so
//! several cases below hammer the same subject and assert the answer never moves.
//!
//! *Uniformity.* A skewed bucket function silently biases every result. The
//! distribution case runs 100,000 synthetic subjects through a configured split and
//! asserts the observed shares land inside a stated tolerance — see
//! `the_observed_split_matches_the_configured_split` for the tolerance and why it is
//! the number it is.
//!
//! Every case runs a real `.tether` script through the binary, so these exercise the
//! registered built-ins rather than the Rust functions behind them.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own source file.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// A 50/50 experiment with a sticky cookie, as a script prelude.
///
/// Declared once because nine cases need the same experiment and an inline copy in
/// each would let them drift apart, which would make a failure ambiguous.
const FIFTY_FIFTY: &str = r#"
fn variant(name, weight) {
    let v = map()
    v.name = name
    v.weight = weight
    v
}

fn split(seed, first, second) {
    let cfg = map()
    cfg.name = "checkout_button"
    cfg.seed = seed
    cfg.sticky_cookie = "ab_checkout"
    cfg.variants = [variant("control", first), variant("green", second)]
    ab_experiment(cfg)
}
"#;

/// Run a script and return its trimmed stdout, asserting it succeeded.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_abtest_{}", std::process::id()));
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

/// Run a script with the shared experiment prelude prepended.
fn run_with_prelude(body: &str) -> String {
    run(&format!("{FIFTY_FIFTY}\n{body}\n"))
}

/// Split stdout into lines for positional assertions.
fn lines(out: &str) -> Vec<String> {
    out.lines().map(|line| line.to_string()).collect()
}

// ---------------------------------------------------------------- construction

#[test]
fn a_valid_experiment_echoes_its_configuration_back() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("checkout_v1", 50, 50)?
    println(exp.name)
    println(exp.seed)
    println(exp.sticky_cookie)
    println(str(exp.variants.len()))
    println(exp.variants[0].name)
    println(str(exp.variants[0].weight))
    println(exp.variants[1].name)
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "checkout_button", "name must round-trip: {out}");
    assert_eq!(got[1], "checkout_v1", "seed must round-trip: {out}");
    assert_eq!(got[2], "ab_checkout", "sticky cookie must round-trip: {out}");
    assert_eq!(got[3], "2", "both variants must survive: {out}");
    assert_eq!(
        got[4], "control",
        "configured order is load-bearing and must be preserved: {out}"
    );
    assert_eq!(got[5], "50", "weight must round-trip: {out}");
    assert_eq!(got[6], "green", "second variant must keep its slot: {out}");
}

#[test]
fn a_missing_sticky_cookie_is_optional_and_reported_as_nil() {
    let out = run(r#"fn main() {
    let v = map()
    v.name = "only"
    v.weight = 100
    let cfg = map()
    cfg.name = "no_cookie"
    cfg.seed = "s1"
    cfg.variants = [v]
    let exp = ab_experiment(cfg)?
    println(str(exp.sticky_cookie == nil))
    println(ab_assign(exp, "anyone")?)
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "true",
        "sticky_cookie is optional and must read as nil when unset: {out}"
    );
    assert_eq!(
        got[1], "only",
        "an experiment without a cookie must still assign: {out}"
    );
}

// ------------------------------------------------------------------ rejections

/// Rejecting a bad sum at construction is the whole point: normalising it would skew
/// traffic for the life of the experiment and nobody would ever find out.
#[test]
fn weights_that_do_not_sum_to_one_hundred_are_rejected_naming_the_sum() {
    let out = run_with_prelude(
        r#"fn main() {
    let low = split("s", 40, 40)
    let high = split("s", 70, 50)
    println(str(low.is_err()))
    println(low.err())
    println(high.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "true", "80 must not be accepted: {out}");
    assert!(
        got[1].contains("100") && got[1].contains("80"),
        "error must state both the required total and the sum seen, got: {}",
        got[1]
    );
    assert!(
        got[2].contains("120"),
        "error must name the observed sum, got: {}",
        got[2]
    );
}

#[test]
fn zero_variants_are_rejected() {
    let out = run(r#"fn main() {
    let cfg = map()
    cfg.name = "empty"
    cfg.seed = "s1"
    cfg.variants = []
    let bad = ab_experiment(cfg)
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "an empty variant list must error: {out}");
    assert!(
        got[1].contains("variants"),
        "error must name the field, got: {}",
        got[1]
    );
}

#[test]
fn a_negative_weight_is_rejected_naming_the_variant() {
    let out = run_with_prelude(
        r#"fn main() {
    let bad = split("s", -10, 110)
    println(str(bad.is_err()))
    println(bad.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "true", "a negative weight must error: {out}");
    assert!(
        got[1].contains("negative") && got[1].contains("control"),
        "error must say what is wrong and which variant, got: {}",
        got[1]
    );
}

/// Duplicates are not merely redundant: an assignment returns a *name*, so two
/// variants sharing one produce results that cannot be told apart.
#[test]
fn duplicate_variant_names_are_rejected() {
    let out = run_with_prelude(
        r#"fn main() {
    let cfg = map()
    cfg.name = "dupes"
    cfg.seed = "s1"
    cfg.variants = [variant("same", 50), variant("same", 50)]
    let bad = ab_experiment(cfg)
    println(str(bad.is_err()))
    println(bad.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "true", "a repeated name must error: {out}");
    assert!(
        got[1].contains("duplicate") && got[1].contains("same"),
        "error must name the repeated variant, got: {}",
        got[1]
    );
}

#[test]
fn a_missing_seed_or_name_is_rejected_naming_the_field() {
    let out = run_with_prelude(
        r#"fn main() {
    let no_seed = map()
    no_seed.name = "x"
    no_seed.variants = [variant("a", 100)]
    let no_name = map()
    no_name.seed = "s"
    no_name.variants = [variant("a", 100)]
    println(ab_experiment(no_seed).err())
    println(ab_experiment(no_name).err())
}"#,
    );
    let got = lines(&out);
    assert!(
        got[0].contains("seed"),
        "must name the missing seed, got: {}",
        got[0]
    );
    assert!(
        got[1].contains("name"),
        "must name the missing name, got: {}",
        got[1]
    );
}

#[test]
fn a_non_integer_weight_is_rejected_rather_than_truncated() {
    // 33.3 truncated to 33 would break the sum check in a way the operator did not
    // write, so a float weight is refused outright.
    let out = run_with_prelude(
        r#"fn main() {
    let cfg = map()
    cfg.name = "floaty"
    cfg.seed = "s1"
    cfg.variants = [variant("a", 50.5), variant("b", 49)]
    let bad = ab_experiment(cfg)
    println(str(bad.is_err()))
    println(bad.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "true", "a float weight must error: {out}");
    assert!(
        got[1].contains("weight"),
        "error must name the field, got: {}",
        got[1]
    );
}

#[test]
fn an_empty_subject_is_rejected() {
    // An empty subject would collapse every visitor onto one bucket, which is a
    // silent way to break an experiment.
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("s1", 50, 50)?
    let bad = ab_assign(exp, "")
    println(str(bad.is_err()))
    println(bad.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "true", "an empty subject must error: {out}");
    assert!(
        got[1].contains("subject"),
        "error must name the subject, got: {}",
        got[1]
    );
}

// ------------------------------------------------------------------- stability

#[test]
fn the_same_subject_gets_the_same_variant_across_many_repeats() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("checkout_v1", 50, 50)?
    let first = ab_assign(exp, "visitor-91af")?
    let mut stable = true
    let mut i = 0
    while i < 2000 {
        if ab_assign(exp, "visitor-91af")? != first { stable = false }
        i = i + 1
    }
    println(str(stable))
    println(first)
}"#,
    );
    let got = lines(&out);
    assert_eq!(
        got[0], "true",
        "assignment must be a pure function of (seed, subject): {out}"
    );
    assert!(
        got[1] == "control" || got[1] == "green",
        "must return a configured variant, got: {}",
        got[1]
    );
}

/// A hash has no hidden state, so a fresh process must reach the same answer. This
/// is the property that a random number generator could not provide, and it is why
/// the implementation uses SHA-256 instead.
#[test]
fn assignment_is_identical_in_a_separate_process() {
    let program = r#"fn main() {
    let exp = split("checkout_v1", 50, 50)?
    println(ab_assign(exp, "visitor-91af")?)
    println(ab_assign(exp, "visitor-2200")?)
    println(str(ab_bucket("checkout_v1", "visitor-91af")))
}"#;
    let first = run_with_prelude(program);
    let second = run_with_prelude(program);
    assert_eq!(
        first, second,
        "two independent runs must agree, or the experiment cannot be analysed"
    );
}

// ------------------------------------------------------------------ the bucket

#[test]
fn bucket_stays_within_bounds_over_many_subjects() {
    let out = run(r#"fn main() {
    let mut low = 10000
    let mut high = -1
    let mut i = 0
    while i < 20000 {
        let b = ab_bucket("bounds_seed", "subject-" + i)
        if b < low { low = b }
        if b > high { high = b }
        i = i + 1
    }
    println(str(low >= 0))
    println(str(high < 10000))
    println(str(low < 20))
    println(str(high > 9980))
}"#);
    let got = lines(&out);
    assert_eq!(got[0], "true", "a bucket must never be negative: {out}");
    assert_eq!(got[1], "true", "a bucket must never reach 10000: {out}");
    assert_eq!(
        got[2], "true",
        "20000 subjects should reach near the bottom of the range: {out}"
    );
    assert_eq!(
        got[3], "true",
        "20000 subjects should reach near the top of the range: {out}"
    );
}

/// The bucket is what makes a weight boundary meaningful, so it must move when the
/// seed does. Otherwise a follow-up experiment inherits the previous one's split and
/// its results are contaminated by the earlier exposure.
#[test]
fn changing_the_seed_reshuffles_buckets() {
    let out = run(r#"fn main() {
    let mut moved = 0
    let mut i = 0
    while i < 1000 {
        let s = "subject-" + i
        if ab_bucket("seed_a", s) != ab_bucket("seed_b", s) { moved = moved + 1 }
        i = i + 1
    }
    println(str(moved))
}"#);
    let moved: i64 = out.trim().parse().expect("count should be an int");
    assert!(
        moved > 990,
        "a different seed must move essentially every bucket, moved only {moved}"
    );
}

#[test]
fn changing_the_seed_changes_the_assignment_for_some_subjects() {
    let out = run_with_prelude(
        r#"fn main() {
    let a = split("seed_a", 50, 50)?
    let b = split("seed_b", 50, 50)?
    let mut differing = 0
    let mut i = 0
    while i < 1000 {
        let s = "subject-" + i
        if ab_assign(a, s)? != ab_assign(b, s)? { differing = differing + 1 }
        i = i + 1
    }
    println(str(differing))
}"#,
    );
    let differing: i64 = out.trim().parse().expect("count should be an int");
    // Two independent 50/50 splits disagree on about half of all subjects. The
    // assertion is deliberately loose — the point is that the seed matters at all,
    // not that it produces one exact count.
    assert!(
        differing > 300 && differing < 700,
        "a reseeded 50/50 split should disagree on roughly half of 1000 subjects, got {differing}"
    );
}

// ---------------------------------------------------------------- distribution

/// Uniformity at the bucket level, which is where modulo bias would appear.
///
/// 100,000 subjects are histogrammed into ten equal decile ranges of 1000 buckets
/// each. Under a uniform hash each decile expects 10,000 subjects with standard
/// deviation `sqrt(100000 * 0.1 * 0.9)` ≈ 95, so the **±400 subject** band asserted
/// below is over 4 standard deviations per bin — loose enough that ten bins do not
/// produce a spurious failure, tight enough to catch a real defect.
///
/// This is the test the naive implementation fails. Folding a 2-byte digest prefix
/// with `% 10000` over-weights buckets `0..5536` by about 16%, which lands roughly
/// 1,600 extra subjects in each of the first five deciles and 1,600 too few in the
/// rest — four times outside the band. `abtest_bucket` avoids it by taking eight
/// bytes and scaling with a `u128` multiply-and-shift instead of dividing at all.
#[test]
fn buckets_are_uniform_across_deciles() {
    let out = run(r#"fn main() {
    let mut counts = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    let mut i = 0
    while i < 100000 {
        let d = ab_bucket("uniformity_v1", "subject-" + i) / 1000
        counts[d] = counts[d] + 1
        i = i + 1
    }
    let mut j = 0
    while j < 10 {
        println(str(counts[j]))
        j = j + 1
    }
}"#);
    let counts: Vec<i64> = lines(&out)
        .iter()
        .map(|line| line.parse().expect("each count should be an int"))
        .collect();
    assert_eq!(counts.len(), 10, "one count per decile: {out}");
    assert_eq!(
        counts.iter().sum::<i64>(),
        100_000,
        "integer division by 1000 must partition 0..10000 into exactly ten \
         deciles, so every subject is counted once: {out}"
    );
    for (decile, count) in counts.iter().enumerate() {
        assert!(
            (count - 10_000).abs() <= 400,
            "decile {decile} got {count} of an expected 10000 (+/-400); a lopsided \
             histogram means the bucket function is biased and every weight \
             boundary is wrong: {counts:?}"
        );
    }
}

/// # The tolerance, and why it is 1 percentage point
///
/// 100,000 subjects are bucketed under a 50/50 split. If the bucket function is
/// uniform, the count in either variant behaves like `Binomial(100000, 0.5)`, whose
/// standard deviation is `sqrt(100000 * 0.5 * 0.5)` ≈ 158 subjects, or 0.158
/// percentage points. A tolerance of **±1 percentage point** is therefore about 6.3
/// standard deviations: wide enough that a uniform hash effectively never trips it,
/// and far narrower than any real defect. The modulo bias described in
/// `abtest_bucket` — folding a 2-byte prefix with `% 10000` — over-weights the low
/// buckets by roughly 16%, which at this sample size is thousands of subjects, well
/// outside the band.
///
/// The counts are not actually random, since the hash is deterministic and the
/// subject list is fixed; the distributional reasoning justifies the *width* of the
/// band for a subject set that behaves like a random sample.
#[test]
fn the_observed_split_matches_the_configured_split() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("distribution_v1", 50, 50)?
    let mut control = 0
    let mut green = 0
    let mut i = 0
    while i < 100000 {
        let v = ab_assign(exp, "subject-" + i)?
        if v == "control" { control = control + 1 } else { green = green + 1 }
        i = i + 1
    }
    println(str(control))
    println(str(green))
}"#,
    );
    let got = lines(&out);
    let control: i64 = got[0].parse().expect("control count should be an int");
    let green: i64 = got[1].parse().expect("green count should be an int");
    assert_eq!(
        control + green,
        100_000,
        "every subject must land in exactly one variant: {out}"
    );
    // 1 percentage point of 100000 subjects.
    let tolerance = 1_000;
    assert!(
        (control - 50_000).abs() <= tolerance,
        "control got {control} of 100000, outside 50% +/- 1pp; a skew this large \
         invalidates every result the experiment would produce"
    );
    assert!(
        (green - 50_000).abs() <= tolerance,
        "green got {green} of 100000, outside 50% +/- 1pp"
    );
}

/// The same tolerance argument, applied to an uneven split. `sqrt(100000*0.7*0.3)`
/// ≈ 145, so ±1pp is still over 6 standard deviations.
#[test]
fn an_uneven_split_is_also_honoured_within_tolerance() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("uneven_v1", 70, 30)?
    let mut control = 0
    let mut i = 0
    while i < 100000 {
        if ab_assign(exp, "subject-" + i)? == "control" { control = control + 1 }
        i = i + 1
    }
    println(str(control))
}"#,
    );
    let control: i64 = out.trim().parse().expect("count should be an int");
    assert!(
        (control - 70_000).abs() <= 1_000,
        "a 70/30 split gave control {control} of 100000, outside 70% +/- 1pp"
    );
}

#[test]
fn a_hundred_zero_split_puts_every_subject_in_the_first_variant() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("parked_v1", 100, 0)?
    let mut off = 0
    let mut i = 0
    while i < 20000 {
        if ab_assign(exp, "subject-" + i)? != "control" { off = off + 1 }
        i = i + 1
    }
    println(str(off))
}"#,
    );
    assert_eq!(
        out.trim(),
        "0",
        "a zero-weight variant owns an empty bucket range and must never be \
         selected, so an experiment can be parked without deleting its variants"
    );
}

/// Raising the first variant's weight from 50 to 60 must move *only* the subjects in
/// buckets `5000..6000`. Anything else migrating would mean a weight change churns
/// visitors who were already exposed, which is the failure the half-open ranges in
/// `abtest_assign` exist to prevent.
#[test]
fn a_weight_change_moves_exactly_the_subjects_at_the_new_boundary() {
    let out = run_with_prelude(
        r#"fn main() {
    let fifty = split("boundary_v1", 50, 50)?
    let sixty = split("boundary_v1", 60, 40)?
    let mut moved = 0
    let mut wrong = 0
    let mut i = 0
    while i < 20000 {
        let s = "subject-" + i
        let b = ab_bucket("boundary_v1", s)
        let before = ab_assign(fifty, s)?
        let after = ab_assign(sixty, s)?
        if before != after {
            moved = moved + 1
            if b < 5000 || b >= 6000 { wrong = wrong + 1 }
            if before != "green" || after != "control" { wrong = wrong + 1 }
        }
        i = i + 1
    }
    println(str(wrong))
    println(str(moved > 0))
}"#,
    );
    let got = lines(&out);
    assert_eq!(
        got[0], "0",
        "only subjects in buckets 5000..6000 may move, and only from green to \
         control: {out}"
    );
    assert_eq!(
        got[1], "true",
        "a 10-point shift over 20000 subjects must move someone: {out}"
    );
}

// --------------------------------------------------------- request integration

/// An existing cookie must win. If the recomputed value were preferred, a
/// mid-experiment weight change would migrate visitors who had already been exposed,
/// and their behaviour would be attributed to a variant they barely saw.
#[test]
fn an_existing_sticky_cookie_overrides_computation() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let subject = "visitor-91af"
    let fresh = ab_assign(exp, subject)?
    let other = if fresh == "control" { "green" } else { "control" }

    let jar = map()
    jar["ab_checkout"] = other
    let req = map()
    req.cookies = jar
    req.subject = subject

    let got = ab_assign_from_request(exp, req)?
    println(got.variant)
    println(other)
    println(got.source)
    println(str(got.set_cookie))
    println(str(got.bucket == nil))
}"#,
    );
    let got = lines(&out);
    assert_eq!(
        got[0], got[1],
        "the cookie must win over a fresh computation: {out}"
    );
    assert_eq!(got[2], "cookie", "source must report the cookie: {out}");
    assert_eq!(
        got[3], "false",
        "a cookie the browser already holds must not be re-set: {out}"
    );
    assert_eq!(
        got[4], "true",
        "bucket must be nil when it did not decide, rather than misleadingly \
         recomputed: {out}"
    );
}

#[test]
fn a_request_with_no_cookie_reports_that_one_should_be_set() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let req = map()
    req.subject = "visitor-91af"
    let got = ab_assign_from_request(exp, req)?
    println(got.source)
    println(str(got.set_cookie))
    println(got.cookie_name)
    println(str(got.variant == ab_assign(exp, "visitor-91af")?))
    println(str(got.bucket == ab_bucket("sticky_v1", "visitor-91af")))
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "computed", "no cookie means a fresh bucket: {out}");
    assert_eq!(
        got[1], "true",
        "the caller must be told to pin the assignment: {out}"
    );
    assert_eq!(
        got[2], "ab_checkout",
        "the configured cookie name must be reported so the caller can set it: {out}"
    );
    assert_eq!(
        got[3], "true",
        "the request path must agree with ab_assign: {out}"
    );
    assert_eq!(got[4], "true", "the reported bucket must be the real one: {out}");
}

#[test]
fn a_sticky_cookie_is_read_from_a_raw_cookie_header() {
    // A handler that has not called cookie_parse must still work.
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let headers = map()
    headers["cookie"] = "theme=dark; ab_checkout=green; sid=abc=="
    let req = map()
    req.headers = headers
    req.subject = "visitor-91af"
    let got = ab_assign_from_request(exp, req)?
    println(got.variant)
    println(got.source)
}"#,
    );
    let got = lines(&out);
    assert_eq!(got[0], "green", "the header cookie must be honoured: {out}");
    assert_eq!(got[1], "cookie", "source must report the cookie: {out}");
}

/// A cookie naming a variant that no longer exists cannot be honoured: reporting it
/// would break analysis just as badly as churning the visitor.
#[test]
fn a_cookie_naming_an_unknown_variant_is_discarded_and_rebucketed() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let jar = map()
    jar["ab_checkout"] = "retired_variant"
    let req = map()
    req.cookies = jar
    req.subject = "visitor-91af"
    let got = ab_assign_from_request(exp, req)?
    println(got.source)
    println(str(got.variant == ab_assign(exp, "visitor-91af")?))
    println(str(got.set_cookie))
}"#,
    );
    let got = lines(&out);
    assert_eq!(
        got[0], "computed",
        "an unknown variant must not be honoured: {out}"
    );
    assert_eq!(got[1], "true", "the subject must be re-bucketed: {out}");
    assert_eq!(
        got[2], "true",
        "the stale cookie must be replaced: {out}"
    );
}

#[test]
fn an_empty_cookie_value_counts_as_absent() {
    // Clearing a cookie by setting it to the empty string is common, so an empty
    // value must not be mistaken for an assignment.
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let jar = map()
    jar["ab_checkout"] = ""
    let req = map()
    req.cookies = jar
    req.subject = "visitor-91af"
    println(ab_assign_from_request(exp, req)?.source)
}"#,
    );
    assert_eq!(
        out.trim(),
        "computed",
        "an empty cookie value must be treated as absent: {out}"
    );
}

#[test]
fn a_request_without_a_usable_cookie_or_subject_is_a_named_error() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let bad = ab_assign_from_request(exp, map())
    println(str(bad.is_err()))
    println(bad.err())
}"#,
    );
    let got = lines(&out);
    assert_eq!(
        got[0], "true",
        "nothing identifies the visitor, so this must error: {out}"
    );
    assert!(
        got[1].contains("subject"),
        "error must name the missing field, got: {}",
        got[1]
    );
}

#[test]
fn the_request_path_is_stable_across_repeats() {
    let out = run_with_prelude(
        r#"fn main() {
    let exp = split("sticky_v1", 50, 50)?
    let req = map()
    req.subject = "visitor-2200"
    let first = ab_assign_from_request(exp, req)?.variant
    let mut stable = true
    let mut i = 0
    while i < 500 {
        if ab_assign_from_request(exp, req)?.variant != first { stable = false }
        i = i + 1
    }
    println(str(stable))
}"#,
    );
    assert_eq!(
        out.trim(),
        "true",
        "the request path must be as stable as ab_assign: {out}"
    );
}

// ----------------------------------------------------------------- type errors

#[test]
fn a_non_map_experiment_or_variant_list_is_rejected_by_type() {
    let out = run(r#"fn main() {
    println(ab_experiment("not a map").err())
    let cfg = map()
    cfg.name = "x"
    cfg.seed = "s"
    cfg.variants = "not a list"
    println(ab_experiment(cfg).err())
}"#);
    let got = lines(&out);
    assert!(
        got[0].contains("map") && got[0].contains("str"),
        "error must name both the expectation and what arrived, got: {}",
        got[0]
    );
    assert!(
        got[1].contains("list"),
        "error must say variants is a list, got: {}",
        got[1]
    );
}

#[test]
fn a_non_map_variant_entry_is_rejected_by_type() {
    let out = run(r#"fn main() {
    let cfg = map()
    cfg.name = "x"
    cfg.seed = "s"
    cfg.variants = [1]
    let bad = ab_experiment(cfg)
    println(str(bad.is_err()))
    println(bad.err())
}"#);
    let got = lines(&out);
    assert_eq!(
        got[0], "true",
        "an int where a variant map belongs must error: {out}"
    );
    assert!(
        got[1].contains("variant 0") && got[1].contains("int"),
        "error must name the offending position and its type, got: {}",
        got[1]
    );
}

/// `ab_bucket` returns a bare int rather than a `Result`, so a bad argument is a
/// hard runtime error the script cannot swallow. It must still name the argument.
#[test]
fn bucket_rejects_a_non_string_argument_with_a_named_runtime_error() {
    let dir = std::env::temp_dir().join(format!("tether_abtest_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!(
        "case_{}.tether",
        CASE.fetch_add(1, Ordering::SeqCst)
    ));
    let source = "fn main() {\n    println(str(ab_bucket(7, \"s\")))\n}\n";
    std::fs::write(&path, source).expect("source should be writable");
    let output = Command::new(env!("CARGO_BIN_EXE_tetherscript"))
        .arg("run")
        .arg(&path)
        .output()
        .expect("tetherscript should run");
    assert!(
        !output.status.success(),
        "an int seed must not be silently accepted"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ab_bucket") && stderr.contains("seed"),
        "error must name the built-in and the argument, got: {stderr}"
    );
}