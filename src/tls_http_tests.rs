//! HTTPS verification through ambient language HTTP APIs.

use crate::interp::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

use super::http_client;
use crate::tls::http_test_server;

#[test]
fn http_get_rejects_an_untrusted_certificate() {
    let (url, server) = http_test_server::spawn();
    let error = http_client::client_request("GET", &url, None, &[]).unwrap_err();
    http_test_server::assert_verify_error(&error);
    server.join().unwrap();
}

#[test]
fn https_server_rejects_invalid_pem_before_binding() {
    let source = r#"https_serve(443, "bad cert", "bad key", fn(request) { request })"#;
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let error = Interpreter::new().run_repl(&program).unwrap_err();
    assert!(error.contains("https_serve: invalid TLS identity"));
}
