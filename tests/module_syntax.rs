use tetherscript::lexer::Lexer;
use tetherscript::parser::Parser;
use tetherscript::token::Token;

#[test]
fn module_words_are_keywords() {
    let tokens = Lexer::new("import export as").tokenize().unwrap();
    assert!(matches!(tokens[0].token, Token::Import));
    assert!(matches!(tokens[1].token, Token::Export));
    assert!(matches!(tokens[2].token, Token::As));
}

#[test]
fn parses_imports_and_exports_as_program_metadata() {
    let source = concat!(
        "import \"./math.tether\" as math\n",
        "fn answer() { math.value() }\n",
        "export answer\n"
    );
    let tokens = Lexer::new(source).tokenize().unwrap();
    let program = Parser::new(tokens).parse_program().unwrap();
    assert_eq!(program.imports[0].path, "./math.tether");
    assert_eq!(program.imports[0].alias, "math");
    assert_eq!(program.exports, ["answer"]);
}
