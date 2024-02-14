use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            statements: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    LetStatement {
        name: String,
        value: Option<Expression>,
    },
    ReturnStatement(Option<Expression>),
    ExpressionStatement(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(isize),
    Prefix(Prefix, Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Prefix {
    BANG,
    MINUS,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Prefix::BANG => write!(f, "!"),
            Prefix::MINUS => write!(f, "-"),
        }
    }
}
