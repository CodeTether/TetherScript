//! Unit tests for terminal key parsing.

use super::key_parse;

#[test]
fn parses_printable_and_control_keys() {
    assert_eq!(key_parse::parse(b"a").unwrap().text.as_deref(), Some("a"));
    let ctrl_c = key_parse::parse(&[3]).unwrap();
    assert_eq!(ctrl_c.text.as_deref(), Some("c"));
    assert!(ctrl_c.ctrl);
}

#[test]
fn parses_common_escape_keys() {
    assert_eq!(key_parse::parse(b"\x1b[A").unwrap().key, "up");
    assert_eq!(key_parse::parse(b"\x1b[B").unwrap().key, "down");
    assert_eq!(key_parse::parse(b"\x1b[3~").unwrap().key, "delete");
}
