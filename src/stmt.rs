use std::fmt::{Display, Write};
use crate::expr::Expr;
use crate::token::Token;

#[derive(Clone)]
pub enum Stmt {
    VarDeclaration(Token, Option<Expr>),
    Print(Expr),
    Expression(Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(Token, Vec<Token>, Vec<Stmt>),
}

impl Display for Stmt {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Stmt::VarDeclaration(t, e) => {
                let mut ret = String::from(format!("VarDeclaration {}", *t).as_str());
                if e.is_some() {
                    ret.write_str(format!("= {}", e.clone().unwrap()).as_str()).expect("");
                }
                fmt.write_str(&*ret)
            }
            Stmt::If(condition, if_body, else_body) => {
                let mut ret = String::from( format!("If {} {}", condition, if_body).as_str());
                if else_body.is_some() {
                    ret.write_str(format!("= {}", else_body.clone().unwrap()).as_str()).expect("");
                }
                fmt.write_str(&*ret)
            }
            Stmt::Expression(e) => fmt.write_str(format!("Expr {}", e).as_str()),
            Stmt::Block(v) => {
                let mut ret = String::from("Block: \n");
                for s in v {
                    ret.write_str(format!("\t{}\n", s).as_str()).expect("");
                }
                fmt.write_str(&*ret)
            }
            Stmt::While(e, s) => {
                fmt.write_str(format!("While [{}] [{}]", e, *s).as_str())
            }
            Stmt::Print(e) => fmt.write_str(format!("Print {}", e).as_str()),
            Stmt::Function(name, _params, _body) => {
                fmt.write_str(format!("fun {}", &name.lexeme).as_str())
            }
            // Stmt::NativeFunction(name, params, body) => {
            //     fmt.write_str(format!("fun {}({})", &name.lexeme, *params.iter().map(|x| *x.lexeme).collect().join(", ")).as_str())
            // }
        }
    }
}

impl Stmt {

}