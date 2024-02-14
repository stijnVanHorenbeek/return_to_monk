use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
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
    EQ,
    NOT_EQ,
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
            Token::ASSIGN => write!(f, "="),
            Token::PLUS => write!(f, "+"),
            Token::MINUS => write!(f, "-"),
            Token::BANG => write!(f, "!"),
            Token::ASTERISK => write!(f, "*"),
            Token::SLASH => write!(f, "/"),
            Token::LT => write!(f, "<"),
            Token::GT => write!(f, ">"),
            Token::COMMA => write!(f, ","),
            Token::SEMICOLON => write!(f, ";"),
            Token::LPAREN => write!(f, "("),
            Token::RPAREN => write!(f, ")"),
            Token::LBRACE => write!(f, "{{"),
            Token::RBRACE => write!(f, "}}"),
            Token::EQ => write!(f, "=="),
            Token::NOT_EQ => write!(f, "!="),
            Token::FUNCTION => write!(f, "fn"),
            Token::LET => write!(f, "let"),
            Token::IF => write!(f, "if"),
            Token::ELSE => write!(f, "else"),
            Token::TRUE => write!(f, "true"),
            Token::FALSE => write!(f, "false"),
            Token::RETURN => write!(f, "return"),
        }
    }
}
