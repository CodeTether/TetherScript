//! Scoped HTTP capability coverage for verified HTTPS.

use crate::interp::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::tls::http_test_server;
use crate::value::Value;

use super::HttpAuthority;

#[test]
fn scoped_http_capability_uses_verified_https() {
    let (url, server) = http_test_server::spawn();
    let source = format!(r#"http.get("{url}").err()"#);
    let tokens = Lexer::new(&source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    let mut interpreter = Interpreter::new();
    let origin = url.trim_end_matches('/').to_string();
    interpreter.grant("http", HttpAuthority::new(vec![origin]));
    let result = interpreter.run_repl(&program).unwrap();
    match result {
        Value::Str(error) => http_test_server::assert_verify_error(&error),
        other => panic!("expected scoped HTTPS error string, got {other:?}"),
    }
    server.join().unwrap();
}
