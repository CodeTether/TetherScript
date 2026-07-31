//! Coverage for the MIME built-ins.
//!
//! These run real `.tether` programs, because the built-ins are only reachable
//! through the interpreter: the table and header parser are private submodules,
//! so a unit test could not see them, and the script surface is what the
//! the reference application port actually consumes.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Tests run in parallel, so each case needs its own file name.
static CASE: AtomicUsize = AtomicUsize::new(0);

fn run_source(src: &str) -> std::process::Output {
    let case = CASE.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tether_mime_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    let path = dir.join(format!("mime_case_{case}.tether"));
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

/// Print `mime_for_path` for one path.
fn type_of(path: &str) -> String {
    stdout_of(&format!(
        "fn main() {{ println(mime_for_path(\"{path}\")) }}\n"
    ))
}

#[test]
fn maps_markup_and_style_extensions() {
    assert_eq!(type_of("index.html"), "text/html; charset=utf-8");
    assert_eq!(type_of("index.htm"), "text/html; charset=utf-8");
    assert_eq!(type_of("site.css"), "text/css; charset=utf-8");
}

#[test]
fn maps_script_and_data_extensions() {
    assert_eq!(type_of("app.js"), "text/javascript; charset=utf-8");
    assert_eq!(type_of("app.mjs"), "text/javascript; charset=utf-8");
    assert_eq!(type_of("data.json"), "application/json");
    assert_eq!(type_of("feed.xml"), "application/xml");
    assert_eq!(type_of("out.wasm"), "application/wasm");
}

#[test]
fn maps_text_document_extensions() {
    assert_eq!(type_of("notes.txt"), "text/plain; charset=utf-8");
    assert_eq!(type_of("README.md"), "text/markdown; charset=utf-8");
    assert_eq!(type_of("rows.csv"), "text/csv; charset=utf-8");
}

#[test]
fn maps_image_extensions() {
    assert_eq!(type_of("logo.png"), "image/png");
    assert_eq!(type_of("photo.jpg"), "image/jpeg");
    assert_eq!(type_of("photo.jpeg"), "image/jpeg");
    assert_eq!(type_of("anim.gif"), "image/gif");
    assert_eq!(type_of("hero.webp"), "image/webp");
    assert_eq!(type_of("favicon.ico"), "image/x-icon");
    assert_eq!(type_of("icon.svg"), "image/svg+xml");
}

#[test]
fn maps_font_extensions() {
    assert_eq!(type_of("face.woff"), "font/woff");
    assert_eq!(type_of("face.woff2"), "font/woff2");
    assert_eq!(type_of("face.ttf"), "font/ttf");
}

#[test]
fn maps_archive_and_media_extensions() {
    assert_eq!(type_of("doc.pdf"), "application/pdf");
    assert_eq!(type_of("bundle.zip"), "application/zip");
    assert_eq!(type_of("clip.mp4"), "video/mp4");
    assert_eq!(type_of("clip.webm"), "video/webm");
    assert_eq!(type_of("song.mp3"), "audio/mpeg");
}

/// The mappings shared with the native static server must match it exactly, or a
/// file would be typed differently depending on which server served it.
#[test]
fn shared_mappings_match_the_native_static_server() {
    let source = std::fs::read_to_string("src/http_static/content_type.rs")
        .expect("static server content_type should be readable");
    for (extension, expected) in [
        ("html", "text/html; charset=utf-8"),
        ("css", "text/css; charset=utf-8"),
        ("json", "application/json"),
        ("svg", "image/svg+xml"),
        ("png", "image/png"),
    ] {
        assert!(
            source.contains(expected),
            "static server no longer maps {extension} to {expected}"
        );
        assert_eq!(type_of(&format!("file.{extension}")), expected);
    }
}

#[test]
fn unknown_extension_falls_back_to_octet_stream() {
    assert_eq!(type_of("archive.xyz"), "application/octet-stream");
}

#[test]
fn missing_extension_falls_back_to_octet_stream() {
    assert_eq!(type_of("LICENSE"), "application/octet-stream");
}

/// A dotfile has no extension; treating `gitignore` as one would guess a type.
#[test]
fn dotfile_has_no_extension_and_does_not_panic() {
    assert_eq!(type_of(".gitignore"), "application/octet-stream");
    assert_eq!(type_of(".env"), "application/octet-stream");
}

#[test]
fn extension_matching_ignores_case() {
    assert_eq!(type_of("LOGO.PNG"), "image/png");
    assert_eq!(type_of("Index.HtMl"), "text/html; charset=utf-8");
}

/// Only the final extension counts, and a directory dot must not leak into it.
#[test]
fn only_the_final_extension_of_the_file_name_counts() {
    assert_eq!(type_of("archive.tar.gz"), "application/octet-stream");
    assert_eq!(type_of("bundle.min.js"), "text/javascript; charset=utf-8");
    assert_eq!(type_of("my.assets/logo"), "application/octet-stream");
}

#[test]
fn parse_splits_the_media_type_and_charset() {
    let src = "fn main() {\n    \
        let parsed = mime_parse(\"text/html; charset=utf-8\")?\n    \
        println(parsed.type + \" \" + parsed.charset)\n}\n";
    assert_eq!(stdout_of(src), "text/html utf-8");
}

#[test]
fn parse_handles_a_quoted_parameter_value() {
    let src = "fn main() {\n    \
        let parsed = mime_parse(\"multipart/form-data; boundary=\\\"a;b c\\\"\")?\n    \
        println(parsed.boundary)\n}\n";
    assert_eq!(stdout_of(src), "a;b c");
}

#[test]
fn parse_lowercases_the_type_and_parameter_names() {
    let src = "fn main() {\n    \
        let parsed = mime_parse(\"TEXT/HTML; CharSet=UTF-8\")?\n    \
        println(parsed.type + \" \" + parsed.charset)\n}\n";
    // The value keeps its original case; only names are normalized.
    assert_eq!(stdout_of(src), "text/html UTF-8");
}

#[test]
fn parse_accepts_a_bare_media_type() {
    let src = "fn main() { println(mime_parse(\"image/png\")?.type) }\n";
    assert_eq!(stdout_of(src), "image/png");
}

#[test]
fn parse_rejects_an_empty_header() {
    let src = "fn main() {\n    \
        let parsed = mime_parse(\"\")\n    \
        println(str(parsed.is_err()))\n    \
        println(parsed.err())\n}\n";
    let out = stdout_of(src);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true", "full output: {out}");
    assert!(
        lines[1].contains("no media type"),
        "error should say what was wrong, got: {}",
        lines[1]
    );
}

#[test]
fn is_text_is_true_for_textual_types() {
    let src = "fn main() {\n    \
        println(str(mime_is_text(\"text/plain; charset=utf-8\")))\n    \
        println(str(mime_is_text(\"application/json\")))\n    \
        println(str(mime_is_text(\"image/svg+xml\")))\n    \
        println(str(mime_is_text(\"application/ld+json\")))\n}\n";
    assert_eq!(stdout_of(src), "true\ntrue\ntrue\ntrue");
}

#[test]
fn is_text_is_false_for_binary_types() {
    let src = "fn main() {\n    \
        println(str(mime_is_text(\"image/png\")))\n    \
        println(str(mime_is_text(\"application/octet-stream\")))\n    \
        println(str(mime_is_text(\"font/woff2\")))\n}\n";
    assert_eq!(stdout_of(src), "false\nfalse\nfalse");
}

/// The three built-ins must compose: a path's type should classify correctly.
#[test]
fn for_path_output_feeds_is_text_and_parse() {
    let src = "fn main() {\n    \
        let kind = mime_for_path(\"page.html\")\n    \
        println(str(mime_is_text(kind)))\n    \
        println(mime_parse(kind)?.type)\n}\n";
    assert_eq!(stdout_of(src), "true\ntext/html");
}

#[test]
fn non_string_argument_is_rejected_by_name() {
    let output = run_source("fn main() { println(mime_for_path(42)) }\n");
    assert!(!output.status.success(), "an int path must not be accepted");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mime_for_path: path"),
        "error should name the parameter, got: {stderr}"
    );
}
