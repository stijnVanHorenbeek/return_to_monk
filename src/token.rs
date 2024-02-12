use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Token {
    ILLEGAL,
    EOF,
    IDENT(String),
    INT(isize),
    // Operators
    ASSIGN,
    PLUS,
    MINUS,
    BANG,
    ASTERISK,
    SLASH,
    LT,
    GT,
    // Delimiters
    COMMA,
    SEMICOLON,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    // Keywords
    FUNCTION,
    LET,
    IF,
    ELSE,
    TRUE,
    FALSE,
    RETURN,
}

pub fn lookup_ident(ident: &str) -> Token {
    match ident {
        "fn" => Token::FUNCTION,
        "let" => Token::LET,
        "if" => Token::IF,
        "else" => Token::ELSE,
        "true" => Token::TRUE,
        "false" => Token::FALSE,
        "return" => Token::RETURN,
        _ => Token::IDENT(ident.into()),
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Token::ILLEGAL => write!(f, "ILLEGAL"),
            Token::EOF => write!(f, "EOF"),
            Token::IDENT(s) => write!(f, "IDENT({})", s),
            Token::INT(i) => write!(f, "INT({})", i),
            Token::ASSIGN => write!(f, "ASSIGN"),
            Token::PLUS => write!(f, "PLUS"),
            Token::MINUS => write!(f, "MINUS"),
            Token::BANG => write!(f, "BANG"),
            Token::ASTERISK => write!(f, "ASTERISK"),
            Token::SLASH => write!(f, "SLASH"),
            Token::LT => write!(f, "LT"),
            Token::GT => write!(f, "GT"),
            Token::COMMA => write!(f, "COMMA"),
            Token::SEMICOLON => write!(f, "SEMICOLON"),
            Token::LPAREN => write!(f, "LPAREN"),
            Token::RPAREN => write!(f, "RPAREN"),
            Token::LBRACE => write!(f, "LBRACE"),
            Token::RBRACE => write!(f, "RBRACE"),
            Token::FUNCTION => write!(f, "FUNCTION"),
            Token::LET => write!(f, "LET"),
            Token::IF => write!(f, "IF"),
            Token::ELSE => write!(f, "ELSE"),
            Token::TRUE => write!(f, "TRUE"),
            Token::FALSE => write!(f, "FALSE"),
            Token::RETURN => write!(f, "RETURN"),
        }
    }
}
