#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::LeftBrace => write!(f, "LeftBrace"),
            TokenType::RightBrace => write!(f, "RightBrace"),
            TokenType::Comma => write!(f, "COMMA"),
            TokenType::Dot => write!(f, "DOT"),
            TokenType::Minus => write!(f, "MINUS"),
            TokenType::Plus => write!(f, "PLUS"),
            TokenType::Semicolon => write!(f, "SEMICOLON"),
            TokenType::Slash => write!(f, "SLASH"),
            TokenType::Star => write!(f, "STAR"),
            TokenType::Bang => write!(f, "BANG"),
            TokenType::BangEqual => write!(f, "BangEqual"),
            TokenType::Equal => write!(f, "EQUAL"),
            TokenType::EqualEqual => write!(f, "EqualEqual"),
            TokenType::Greater => write!(f, "GREATER"),
            TokenType::GreaterEqual => write!(f, "GreaterEqual"),
            TokenType::Less => write!(f, "LESS"),
            TokenType::LessEqual => write!(f, "LessEqual"),
            TokenType::Identifier => write!(f, "IDENTIFIER"),
            TokenType::String => write!(f, "STRING"),
            TokenType::Number => write!(f, "NUMBER"),
            TokenType::And => write!(f, "AND"),
            TokenType::Class => write!(f, "CLASS"),
            TokenType::Else => write!(f, "ELSE"),
            TokenType::False => write!(f, "FALSE"),
            TokenType::Fun => write!(f, "FUN"),
            TokenType::For => write!(f, "FOR"),
            TokenType::If => write!(f, "IF"),
            TokenType::Nil => write!(f, "NIL"),
            TokenType::Or => write!(f, "OR"),
            TokenType::Print => write!(f, "PRINT"),
            TokenType::Return => write!(f, "RETURN"),
            TokenType::Super => write!(f, "SUPER"),
            TokenType::This => write!(f, "THIS"),
            TokenType::True => write!(f, "TRUE"),
            TokenType::Var => write!(f, "VAR"),
            TokenType::While => write!(f, "WHILE"),
            TokenType::EOF => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
    pub col: usize,
}

impl Token {

}

impl std::fmt::Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(" Type: ").expect("");
        fmt.write_str(self.token_type.to_string().as_str()).expect("");
        fmt.write_str(" lexeme: ").expect("");
        fmt.write_str(self.lexeme.as_str()).expect("");
        fmt.write_str(" Literal: ").expect("");
        fmt.write_str(self.literal.to_string().as_str()).expect("");
        fmt.write_str(" line: ").expect("");
        fmt.write_str(self.line.to_string().as_str()).expect("");
        fmt.write_str(":").expect("");
        fmt.write_str(self.col.to_string().as_str()).expect("");
        Ok(())
    }
}


#[derive(Debug, Clone)]
pub enum Literal {
    Null,
    Identifier(String),
    String(String),
    Number(f64),
    True,
    False,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Literal::Null => {fmt.write_str(" Null")},
            Literal::Identifier(s) => {fmt.write_str(format!("{}", s).as_str())}
            Literal::String(s) => {fmt.write_str(format!("{}", s).as_str())}
            Literal::Number(n) => {fmt.write_str(format!("{}", n).as_str())}
            Literal::True => {fmt.write_str("True")}
            Literal::False => {fmt.write_str("False")}
        }.expect("");
        Ok(())
    }
}