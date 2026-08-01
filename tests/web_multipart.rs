//! Integration coverage for the multipart/form-data built-ins.
//!
//! The assertions that matter most are byte-exactness of a part body and the
//! rejection of a truncated body. A parser that keeps the delimiter's leading
//! CRLF corrupts every upload by two bytes while still producing a file that
//! opens, so `file_part_body_has_no_trailing_crlf` pins the exact length.
//!
//! tetherscript strings interpret `\r` and `\n`, so every body below spells its
//! CRLFs explicitly rather than relying on source line endings.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script and return its stdout.
fn run(source: &str) -> String {
    let dir = std::env::temp_dir().join(format!("tether_multipart_{}", std::process::id()));
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
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Stdout as trimmed lines, so each assertion targets one printed value.
fn lines(source: &str) -> Vec<String> {
    run(source).lines().map(|line| line.to_string()).collect()
}

#[test]
fn single_text_field_parses() {
    let out = lines(
        r#"
fn main() {
    let body = "--X\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nHello\r\n--X--\r\n"
    let parts = multipart_parse(body, "X").unwrap()
    println(str(parts.len()))
    println(parts[0].name)
    println(parts[0].body)
    println(str(parts[0].filename == nil))
    println(str(parts[0].content_type == nil))
}
"#,
    );
    assert_eq!(out[0], "1", "one part expected: {out:?}");
    assert_eq!(out[1], "title");
    assert_eq!(out[2], "Hello");
    assert_eq!(out[3], "true", "a text field has no filename");
    assert_eq!(out[4], "true", "a text field declares no content type");
}

#[test]
fn two_fields_parse_in_order() {
    let out = lines(
        r#"
fn main() {
    let body = "--B\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\none\r\n--B\r\nContent-Disposition: form-data; name=\"b\"\r\n\r\ntwo\r\n--B--\r\n"
    let parts = multipart_parse(body, "B").unwrap()
    println(str(parts.len()))
    println(parts[0].name + "=" + parts[0].body)
    println(parts[1].name + "=" + parts[1].body)
}
"#,
    );
    assert_eq!(out[0], "2", "two parts expected: {out:?}");
    assert_eq!(out[1], "a=one");
    assert_eq!(out[2], "b=two");
}

#[test]
fn file_part_exposes_filename_and_content_type() {
    let out = lines(
        r#"
fn main() {
    let body = "--Z\r\nContent-Disposition: form-data; name=\"clip\"; filename=\"a b.mp4\"\r\nContent-Type: video/mp4\r\n\r\nDATA\r\n--Z--\r\n"
    let parts = multipart_parse(body, "Z").unwrap()
    println(parts[0].name)
    println(parts[0].filename)
    println(parts[0].content_type)
    println(parts[0].body)
}
"#,
    );
    assert_eq!(out[0], "clip");
    assert_eq!(
        out[1], "a b.mp4",
        "quoted filename with a space must survive"
    );
    assert_eq!(out[2], "video/mp4");
    assert_eq!(out[3], "DATA");
}

/// A body line that merely resembles the boundary must not split the part.
#[test]
fn body_text_resembling_the_boundary_does_not_split() {
    let out = lines(
        r#"
fn main() {
    let body = "--Q\r\nContent-Disposition: form-data; name=\"note\"\r\n\r\nsee --Q inline\r\nand --Qmore\r\n--Q--\r\n"
    let parts = multipart_parse(body, "Q").unwrap()
    println(str(parts.len()))
    println(parts[0].body.replace("\r\n", "|"))
}
"#,
    );
    assert_eq!(out[0], "1", "a look-alike must not create a part: {out:?}");
    assert_eq!(
        out[1], "see --Q inline|and --Qmore",
        "inner text must be preserved verbatim"
    );
}

/// The delimiter's leading CRLF belongs to the delimiter, not to the body.
#[test]
fn file_part_body_has_no_trailing_crlf() {
    let out = lines(
        r#"
fn main() {
    let body = "--W\r\nContent-Disposition: form-data; name=\"f\"; filename=\"f.bin\"\r\n\r\nabc\r\n--W--\r\n"
    let parts = multipart_parse(body, "W").unwrap()
    println(str(parts[0].body.len()))
    println(str(parts[0].body == "abc"))
}
"#,
    );
    assert_eq!(
        out[0], "3",
        "body must be exactly 3 bytes; 5 means the trailing CRLF leaked in"
    );
    assert_eq!(out[1], "true");
}

/// An empty body is legal and must stay empty rather than absorbing the CRLF.
#[test]
fn empty_part_body_stays_empty() {
    let out = lines(
        r#"
fn main() {
    let body = "--E\r\nContent-Disposition: form-data; name=\"blank\"\r\n\r\n\r\n--E--\r\n"
    let parts = multipart_parse(body, "E").unwrap()
    println(str(parts[0].body.len()))
}
"#,
    );
    assert_eq!(out[0], "0", "an empty field must decode to zero bytes");
}

#[test]
fn missing_final_delimiter_is_a_named_error() {
    let out = lines(
        r#"
fn main() {
    let body = "--T\r\nContent-Disposition: form-data; name=\"x\"\r\n\r\ntruncated"
    let bad = multipart_parse(body, "T")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    assert_eq!(out[0], "true", "a truncated body must not parse");
    assert!(
        out[1].contains("closing"),
        "error should name the missing closing delimiter, got: {}",
        out[1]
    );
}

#[test]
fn field_lookup_finds_a_present_name() {
    let out = lines(
        r#"
fn main() {
    let body = "--F\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nBin Day\r\n--F--\r\n"
    let parts = multipart_parse(body, "F").unwrap()
    println(multipart_field(parts, "title").unwrap())
}
"#,
    );
    assert_eq!(out[0], "Bin Day");
}

#[test]
fn field_lookup_names_an_absent_field() {
    let out = lines(
        r#"
fn main() {
    let body = "--F\r\nContent-Disposition: form-data; name=\"title\"\r\n\r\nBin Day\r\n--F--\r\n"
    let parts = multipart_parse(body, "F").unwrap()
    let bad = multipart_field(parts, "missing")
    println(str(bad.is_err()))
    println(bad.err())
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("missing"),
        "error should name the absent field, got: {}",
        out[1]
    );
}

#[test]
fn boundary_is_extracted_from_the_header() {
    let out = lines(
        r#"
fn main() {
    println(multipart_boundary("multipart/form-data; boundary=abc123").unwrap())
    println(multipart_boundary("multipart/form-data; boundary=\"quoted-1\"").unwrap())
}
"#,
    );
    assert_eq!(out[0], "abc123");
    assert_eq!(out[1], "quoted-1", "quotes must be stripped");
}

#[test]
fn header_without_a_boundary_is_a_named_error() {
    let out = lines(
        r#"
fn main() {
    let bad = multipart_boundary("multipart/form-data")
    println(str(bad.is_err()))
    println(bad.err())
    let blank = multipart_boundary("multipart/form-data; boundary=")
    println(str(blank.is_err()))
}
"#,
    );
    assert_eq!(out[0], "true");
    assert!(
        out[1].contains("boundary"),
        "error should name the boundary parameter, got: {}",
        out[1]
    );
    assert_eq!(out[2], "true", "an empty boundary must be rejected");
}
