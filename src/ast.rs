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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Statement {
    LetStatement { name: String, value: Expression },
    ReturnStatement(Expression),
    BlockStatement(Vec<Statement>),
    ExpressionStatement(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Statement::LetStatement { name, value } => {
                write!(f, "let {} = {};", name, value)
            }
            Statement::ReturnStatement(value) => {
                write!(f, "return {};", value)
            }
            Statement::BlockStatement(statements) => {
                let mut result = String::new();
                for statement in statements {
                    result.push_str(&format!("{}", statement));
                }
                write!(f, "{}", result)
            }
            Statement::ExpressionStatement(expression) => write!(f, "{}", expression),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Identifier(String),
    IntegerLiteral(isize),
    BooleanLiteral(bool),
    If {
        condition: Box<Expression>,
        consequence: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    FunctionLiteral {
        parameters: Vec<String>,
        body: Box<Statement>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Prefix(Prefix, Box<Expression>),
    Infix(Infix, Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::IntegerLiteral(value) => write!(f, "{}", value),
            Expression::BooleanLiteral(value) => write!(f, "{}", value),
            Expression::Prefix(operator, right) => write!(f, "({}{})", operator, right),
            Expression::Infix(operator, left, right) => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expression::If {
                condition,
                consequence,
                alternative,
            } => {
                let mut result = String::new();
                result.push_str(&format!("if {} {}", condition, consequence));
                if let Some(alternative) = alternative {
                    result.push_str(&format!("else {}", alternative));
                }
                write!(f, "{}", result)
            }
            Expression::FunctionLiteral { parameters, body } => {
                let mut result = String::new();
                result.push_str("fn(");
                for (i, parameter) in parameters.iter().enumerate() {
                    if i == 0 {
                        result.push_str(&format!("{}", parameter));
                    } else {
                        result.push_str(&format!(", {}", parameter));
                    }
                }
                result.push_str(&format!(") {}", body));
                write!(f, "{}", result)
            }
            Expression::Call {
                function,
                arguments,
            } => {
                let mut result = String::new();
                result.push_str(&format!("{}", function));
                result.push_str("(");
                for (i, argument) in arguments.iter().enumerate() {
                    if i == 0 {
                        result.push_str(&format!("{}", argument));
                    } else {
                        result.push_str(&format!(", {}", argument));
                    }
                }
                result.push_str(")");
                write!(f, "{}", result)
            }
        }
    }
}

#[allow(non_camel_case_types)]
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
