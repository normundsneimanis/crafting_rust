use std::collections::HashMap;
use crate::token::Literal;
use crate::token::TokenType;
use crate::token::Token;

pub struct Scanner {
    source: Vec<u8>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    last_line_start: usize,
    col: usize,
    had_error: bool,
    keywords: HashMap<String, TokenType>,
}


impl Default for Scanner {
    fn default() -> Scanner {
        Scanner {
            source: vec![],
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            last_line_start: 0,
            col: 0,
            had_error: false,
            keywords: HashMap::from([
                (String::from("and"), TokenType::And),
                (String::from("class"), TokenType::Class),
                (String::from("else"), TokenType::Else),
                (String::from("false"), TokenType::False),
                (String::from("for"), TokenType::For),
                (String::from("fun"), TokenType::Fun),
                (String::from("if"), TokenType::If),
                (String::from("nil"), TokenType::Nil),
                (String::from("or"), TokenType::Or),
                (String::from("print"), TokenType::Print),
                (String::from("return"), TokenType::Return),
                (String::from("super"), TokenType::Super),
                (String::from("this"), TokenType::This),
                (String::from("true"), TokenType::True),
                (String::from("var"), TokenType::Var),
                (String::from("while"), TokenType::While),
            ]),
        }
    }
}

impl Scanner {
    pub fn set_source(&mut self, source: &String) {
        self.source = source.clone().into_bytes();
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }
        self.add_token_null(TokenType::EOF);

        let tokens = self.tokens.clone();
        tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token_null(TokenType::LeftParen),
            ')' => self.add_token_null(TokenType::RightParen),
            '{' => self.add_token_null(TokenType::LeftBrace),
            '}' => self.add_token_null(TokenType::RightBrace),
            ',' => self.add_token_null(TokenType::Comma),
            '.' => self.add_token_null(TokenType::Dot),
            '-' => self.add_token_null(TokenType::Minus),
            '+' => self.add_token_null(TokenType::Plus),
            ';' => self.add_token_null(TokenType::Semicolon),
            '*' => self.add_token_null(TokenType::Star),
            '!' => {
                if self.match_next('=') {
                    self.add_token_null(TokenType::BangEqual);
                } else {
                    self.add_token_null(TokenType::Bang)
                }}
            '=' => {
                if self.match_next('=') {
                    self.add_token_null(TokenType::EqualEqual);
                } else {
                    self.add_token_null(TokenType::Equal)
                }}
            '<' => {
                if self.match_next('=') {
                    self.add_token_null(TokenType::LessEqual);
                } else {
                    self.add_token_null(TokenType::Less)
                }}
            '>' => {
                if self.match_next('=') {
                    self.add_token_null(TokenType::GreaterEqual);
                } else {
                    self.add_token_null(TokenType::Greater)
                }}
            '/' => {
                if self.peek() == '/' {
                    self.current += 1;
                    while self.peek() != '\n' {
                        self.current += 1;
                    }
                } else if self.peek() == '*' {
                    self.current += 1;
                    while self.peek() != '*' && self.peek_next() != '/' {
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.col = 0;
                            self.last_line_start = self.current;
                        }
                        self.current += 1;
                    }
                    self.current += 2;
                } else {
                    self.add_token_null(TokenType::Slash);
                }
            }
            ' ' => {},
            '\r' => {},
            '\t' => {},
            '\n' => {self.line += 1; self.col = 0; self.last_line_start = self.current;}
            '"' => {self.string()}
            'o' => {
                if self.match_next('r') {
                    self.add_token_null(TokenType::Or);
                }
            }
             _ => {
                 if self.match_digit(c) {
                     self.number();
                 } else if self.match_alpha(c) {
                     self.identifier()
                 } else {
                     self.error(self.line, String::from(format!("Unexpected character: {}", c)))
                 }
             },
        }
    }

    pub fn had_error(&self) -> bool {
        self.had_error
    }

    fn match_alpha(&mut self, c: char) -> bool {
        if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' {
            true
        } else { false }
    }

    fn match_alphanumeric(&mut self, c: char) -> bool {
        self.match_alpha(c) || self.match_digit(c)
    }

    fn identifier(&mut self) {
        while self.match_alphanumeric(self.peek()) {
            self.advance();
        }
        let text = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap();
        if self.keywords.contains_key(&text) {
            self.add_token_null(*self.keywords.get(&text).unwrap());
        } else {
            self.add_token(TokenType::Identifier, Literal::Identifier(text))
        }
    }

    fn number(&mut self) {
        while self.match_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.match_digit(self.peek_next()) {
            self.advance();
            while self.match_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(TokenType::Number,
                       Literal::Number(String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap().parse::<f64>().unwrap())
        );
    }

    fn match_digit(&mut self, value: char) -> bool {
        value >= '0' && value <= '9'
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.col = 0
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error(self.line, String::from("Unterminated string"));
            return;
        }

        self.advance();

        let value = String::from_utf8(self.source[self.start+1..self.current-1].to_vec()).unwrap();
        self.add_token(TokenType::String, Literal::String(value));
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1] as char
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current] as char
        }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] as char != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn report(&mut self, line: usize, where_: String, message: String) {
        eprintln!("line {}: Error {}: {}", line, where_, message);
        self.had_error = true;
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, String::from(""), message);
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current] as char;
        self.current += 1;
        c
    }

    fn add_token_null(&mut self, token: TokenType) {
        self.add_token(token, Literal::Null);
    }

    fn add_token(&mut self, token: TokenType, literal: Literal) {
        let text = String::from_utf8(self.source[self.start..self.current].to_vec()).unwrap();
        self.tokens.push(Token{token_type: token, lexeme: text, literal, line: self.line, col: self.current - self.last_line_start})
    }
}