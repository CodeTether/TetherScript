//! Keyword recognition for the tetherscript lexer.

use crate::token::Token;

pub(crate) fn token(word: &str) -> Option<Token> {
    Some(match word {
        "fn" => Token::Fn,
        "let" => Token::Let,
        "mut" => Token::Mut,
        "move" => Token::Move,
        "if" => Token::If,
        "else" => Token::Else,
        "while" => Token::While,
        "for" => Token::For,
        "in" => Token::In,
        "return" => Token::Return,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "nil" => Token::Nil,
        "panic" => Token::Panic,
        "async" => Token::Async,
        "await" => Token::Await,
        "spawn" => Token::Spawn,
        "join" => Token::Join,
        "import" => Token::Import,
        "export" => Token::Export,
        "as" => Token::As,
        _ => return None,
    })
}
