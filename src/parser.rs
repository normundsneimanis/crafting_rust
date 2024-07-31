use std::vec;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};


#[derive(Debug)]
pub enum ParseError {
    ParseError {
        expected: TokenType,
        found: TokenType,
        message: String,
        line: usize,
        col: usize,
    },
    ExpectedExpression {
        expected: Vec<TokenType>,
        found: TokenType,
        line: usize,
        col: usize,
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::ParseError{expected, found ,message, line, col} =>
                write!(f, "Unexpected token {:?}, expected {:?}: {} at line: {}:{}.", found, expected, message, line, col),
            ParseError::ExpectedExpression{expected, found, line, col} =>
                write!(f, "Unexpected expression {}. Expected {:?} at line: {}:{}.", found, expected, line, col)
        }
    }
}

/*
expression     → equality ;
equality       → comparison ( ( "!=" | "==" ) comparison )* ;
comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
term           → factor ( ( "-" | "+" ) factor )* ;
factor         → unary ( ( "/" | "*" ) unary )* ;
unary          → ( "!" | "-" ) unary
               | primary ;
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
 */


#[derive(Default)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Vec<Stmt>
}


impl Parser {
    pub fn parse(&mut self, tokens: Vec<Token>) -> Vec<Stmt> {
        self.tokens = tokens;
        self.current = 0;
        self.statements = vec![];

        while !self.is_at_end() {
             match self.declaration_or_stmt() {
                 Ok(res) => self.statements.push(res),
                 Err(err) => {self.synchronize(); eprintln!("{}", err)},
             }
        }

        self.statements.clone()
    }

    fn declaration_or_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.match_(vec![TokenType::Fun]) {
            return self.fun_declaration(String::from("function"));
        }
        if self.match_(vec![TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn make_error(&self, expected: TokenType, message: String) -> ParseError {
        ParseError::ParseError {
            expected,
            found: self.peek().token_type,
            message,
            line: self.tokens[self.current-1].line,
            col: self.tokens[self.current-1].col
        }
    }

    fn fun_declaration(&mut self, kind: String) -> Result<Stmt, ParseError> {
        let name = self.consume_(TokenType::Identifier,
                                       String::from(format!("Expecting {} name.", kind).as_str()))?;
        self.consume_(TokenType::LeftParen, String::from(format!("Expect '(' after {} name.", kind).as_str()))?;
        let mut parameters: Vec<Token> = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(self.make_error(TokenType::RightParen, String::from("Too many arguments (>=255).")));
                }

                parameters.push(self.consume_(TokenType::Identifier, String::from("Expect parameter name."))?);
                if !self.match_(vec![TokenType::Comma]) {
                    break
                }
            }
        }
        self.consume_(TokenType::RightParen, String::from("Expect ')' after parameters."))?;
        self.consume_(TokenType::LeftBrace, String::from(format!("Expect '{{' before {} body.", kind).as_str()))?;
        let body = self.block()?;
        Ok(Stmt::Function(name, parameters, body))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_(vec![TokenType::Print]) {
            return self.print_statement();
        };
        if self.match_(vec![TokenType::While]) {
            return self.while_statement();
        };
        if self.match_(vec![TokenType::For]) {
            return self.for_statement();
        };
        if self.match_(vec![TokenType::LeftBrace]) {
            return self.block_statement();
        };
        if self.match_(vec![TokenType::If]) {
            return self.if_statement();
        };
        self.expression_statement()
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_(TokenType::LeftParen, String::from("Expect '(' after 'while'."))?;
        let condition = self.expression()?;
        self.consume_(TokenType::RightParen, String::from("Expect ')' after condition."))?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_(TokenType::LeftParen, String::from("Expect '(' after 'for'."))?;
        let initializer: Option<Stmt>;
        if self.match_(vec![TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_(vec![TokenType::Var]) {
            initializer = self.var_declaration().ok();
        } else {
            initializer = self.expression_statement().ok();
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = self.expression().ok();
        }

        self.consume_(TokenType::Semicolon, String::from("Expect ';' after loop condition."))?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = self.expression().ok();
        }
        self.consume_(TokenType::RightParen, String::from("Expect ')' after for clauses."))?;

        let mut body = self.statement()?;

        if increment.is_some() {
            body = Stmt::Block(vec![body, Stmt::Expression(increment.unwrap())])
        }

        if !condition.is_some() {
            condition = Some(Expr::Literal(Literal::True));
        }

        body = Stmt::While(condition.unwrap(), Box::new(body));

        if initializer.is_some() {
            body = Stmt::Block(vec![initializer.unwrap(), body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume_(TokenType::LeftParen, String::from("Expect '(' after 'if'."))?;
        let condition = self.expression()?;
        let then = Box::new(self.statement()?);
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.match_(vec![TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(condition, then, else_branch))
    }

    fn block_statement(&mut self) -> Result<Stmt, ParseError> {
        Ok(Stmt::Block(self.block()?))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts: Vec<Stmt> = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration_or_stmt()?)
        }

        self.consume_(TokenType::RightBrace, String::from("Expect '}' after block."))?;

        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;
        self.consume_(TokenType::Semicolon, String::from("Expect ';' after value."))?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume_(TokenType::Semicolon, String::from("Expect ';' after expression."))?;
        Ok(Stmt::Expression(expr))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_(TokenType::Identifier, String::from("Expected variable name"))?;
        let mut initializer: Option<Expr> = None;
        if self.match_(vec![TokenType::Equal]) {
            initializer = match self.expression() {
                Ok(e) => Some(e),
                Err(_) => None,
            };
        }

        self.consume_(TokenType::Semicolon, String::from("Expected ';' after variable declaration"))?;
        Ok(Stmt::VarDeclaration(name, initializer))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_(vec![TokenType::Equal]) {
            let right = self.assignment()?;

            if let Expr::Variable(l) = &expr {
                if TokenType::Identifier == l.token_type {
                    return Ok(Expr::Assign(l.clone(), Box::new(right)));
                }
            }
            return Err(ParseError::ParseError {
                expected: TokenType::Var,
                found: TokenType::Nil,
                message: String::from("Invalid assignment target."),
                line: self.tokens[self.current-1].line,
                col: self.tokens[self.current-3].col
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        while self.match_(vec![TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.match_(vec![TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        if self.match_(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        if self.match_(vec![TokenType::Greater, TokenType::GreaterEqual,
                            TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right = Box::new(self.term()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.match_(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = Box::new(self.factor()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.match_(vec![TokenType::Star, TokenType::Slash]) {
            let operator = self.previous();
            let right = Box::new(self.unary()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = Box::new(self.unary()?);
            return Ok(Expr::Unary(operator, right));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments: Vec<Expr> = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParseError::ParseError {
                        expected: TokenType::Var,
                        found: TokenType::Nil,
                        message: String::from("Too many arguments (>=255)."),
                        line: self.tokens[self.current-1].line,
                        col: self.tokens[self.current-1].col
                    });
                }
                arguments.push(self.expression()?);
                if self.match_(vec![TokenType::Comma]) {
                    continue
                } else {
                    break
                }
            }
        }

        let paren = self.consume_(TokenType::RightParen, String::from("Expect ')' after arguments."))?;

        Ok(Expr::Call(Box::new(callee), paren, arguments))
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_(vec![TokenType::False]) {
            return Ok(Expr::Literal(Literal::False));
        } else if self.match_(vec![TokenType::True]) {
            return Ok(Expr::Literal(Literal::True));
        } else if self.match_(vec![TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Null));
        } else if self.match_(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal(self.previous().literal));
        } else if self.match_(vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        } else if self.match_(vec![TokenType::LeftParen]) {
            let expr: Box<Expr> = Box::new(self.expression()?);
            self.consume_(TokenType::RightParen, String::from("Expect ')' after expression."))?;
            return Ok(Expr::Grouping(expr));
        } else {
            eprintln!("Failed in Parser::primary()");
            let last_token = self.peek();
            Err(ParseError::ExpectedExpression {
                expected: vec![TokenType::False, TokenType::True, TokenType::Nil, TokenType::Number,
                               TokenType::String, TokenType::LeftParen],
                found: last_token.token_type,
                line: last_token.line,
                col: last_token.col,
            })
        }
    }

    fn match_(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types.iter() {
            if self.matches(*token_type) {
                return true;
            }
        }
        false
    }

    fn matches(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        return if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume_(&mut self, token_type: TokenType, message: String) -> Result<Token, ParseError> {
        return if self.check(token_type) {
            Ok(self.advance())
        } else {
            eprintln!("Parse Error: {}", message);
            let last_token = self.peek();
            Err(ParseError::ParseError {
                expected: token_type,
                found: last_token.token_type,
                message,
                line: last_token.line,
                col: last_token.col,
            })
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class => {}
                TokenType::Var => {},
                TokenType::Fun => {}
                TokenType::For => {}
                TokenType::If => {}
                TokenType::While => {}
                TokenType::Print => {},
                TokenType::Return => {return},
                TokenType::Semicolon => {},
                _ => {eprintln!("Unexpected token_type in Parser::synchronize(): {}", self.peek().token_type)}
            }
            self.advance();
        }
    }
}