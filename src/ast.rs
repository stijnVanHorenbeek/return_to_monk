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

impl Display for Program {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut result = String::new();
        for statement in &self.statements {
            result.push_str(&format!("{}", statement));
        }
        write!(f, "{}", result)
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

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::LetStatement { name, value } => {
                if let Some(value) = value {
                    write!(f, "let {} = {};", name, value)
                } else {
                    write!(f, "let {};", name)
                }
            }
            Statement::ReturnStatement(value) => {
                if let Some(value) = value {
                    write!(f, "return {};", value)
                } else {
                    write!(f, "return;")
                }
            }
            Statement::ExpressionStatement(expression) => write!(f, "{}", expression),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(isize),
    Prefix(Prefix, Box<Expression>),
    Infix(Infix, Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::IntegerLiteral(value) => write!(f, "{}", value),
            Expression::Prefix(operator, right) => write!(f, "({}{})", operator, right),
            Expression::Infix(operator, left, right) => {
                write!(f, "({} {} {})", left, operator, right)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Infix {
    PLUS,
    MINUS,
    ASTERISK,
    SLASH,
    EQ,
    NOT_EQ,
    LT,
    GT,
}

impl Display for Infix {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Infix::PLUS => write!(f, "+"),
            Infix::MINUS => write!(f, "-"),
            Infix::ASTERISK => write!(f, "*"),
            Infix::SLASH => write!(f, "/"),
            Infix::EQ => write!(f, "=="),
            Infix::NOT_EQ => write!(f, "!="),
            Infix::LT => write!(f, "<"),
            Infix::GT => write!(f, ">"),
        }
    }
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
