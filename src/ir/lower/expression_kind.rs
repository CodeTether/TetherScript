use crate::ast::Expr;

pub(super) fn of(expression: &Expr) -> &'static str {
    match expression {
        Expr::Bytes(_) => "bytes",
        Expr::Unary { .. } => "unary",
        Expr::List(_) => "list",
        Expr::Index { .. } => "index",
        Expr::Field { .. } => "field",
        Expr::Method { .. } => "method",
        Expr::Borrow(_) => "borrow",
        Expr::BorrowMut(_) => "mutable borrow",
        Expr::If { .. } => "if",
        Expr::While { .. } => "while",
        Expr::For { .. } => "for",
        Expr::Block(_) => "block",
        Expr::Fn { .. } => "closure",
        Expr::Return(_) => "return",
        Expr::Panic(_) => "panic",
        Expr::AsyncFn { .. } => "async function",
        Expr::Await(_) => "await",
        Expr::Spawn(_) => "spawn",
        Expr::Join(_) => "join",
        Expr::Try(_) => "try",
        Expr::StringInterp(_) => "string interpolation",
        _ => "unknown",
    }
}
