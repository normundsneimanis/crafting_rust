use crate::token::{Literal, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    // This,
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    Variable(Token), // Get contents of variable
    Assign(Token, Box<Expr>),  // Assign value to variable
    Logical(Box<Expr>, Token, Box<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Literal(l) => {fmt.write_str(format!("{}", l.to_string()).as_str())},
            Expr::Unary(t,e ) => {fmt.write_str(format!("({}{})", t.lexeme, e.to_string()).as_str())}
            Expr::Binary(e1, t, e2) => {
                fmt.write_str(format!("({} {} {})", e1.to_string(), t.lexeme, e2.to_string()).as_str())
            }
            Expr::Variable(t) => {
                fmt.write_str(format!("(variable: {})", t.lexeme).as_str())
            }
            Expr::Logical(e1, t, e2) => {
                fmt.write_str(format!("({} {} {})", e1.to_string(), t.lexeme, e2.to_string()).as_str())
            }
            Expr::Assign(t, e) => {
                fmt.write_str(format!("({} {} {})", t.to_string(), *t, e.to_string()).as_str())
            }
            Expr::Grouping(l) => {fmt.write_str(format!("({})", l.to_string().as_str()).as_str())},
            Expr::Call(_callee, paren, _arguments) => {fmt.write_str(format!("fun {}()", paren.lexeme).as_str())},
        }.expect("");
        Ok(())
    }
}
