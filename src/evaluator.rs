#![allow(dead_code)]

use crate::ast::*;
use anyhow::{Error, Result};
use std::fmt::Display;

pub enum Object {
    Integer(isize),
    Boolean(bool),
    Null,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::Null => write!(f, "null"),
        }
    }
}

pub fn eval(program: Program) -> Result<Object> {
    eval_statements(&program.statements)
}

fn eval_statements(statements: &[Statement]) -> Result<Object, anyhow::Error> {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement)?;
    }

    Ok(result)
}

fn eval_statement(statement: &Statement) -> Result<Object, anyhow::Error> {
    match statement {
        Statement::LetStatement { name: _, value } => {
            let obj = eval_expression(value);
            obj
        }
        Statement::ExpressionStatement(expression) => eval_expression(expression),
        _ => Err(Error::msg("not implemented")),
    }
}

fn eval_expression(expression: &Expression) -> Result<Object, anyhow::Error> {
    match expression {
        Expression::IntegerLiteral(value) => Ok(Object::Integer(*value)),
        Expression::BooleanLiteral(value) => Ok(Object::Boolean(*value)),
        _ => Err(Error::msg("not implemented")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![("5", 5), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        eval(program).unwrap()
    }

    fn test_integer_object(obj: Object, expected: isize) {
        match obj {
            Object::Integer(value) => {
                assert_eq!(value, expected);
            }
            _ => {
                panic!("Object is not Integer. got={}", obj);
            }
        }
    }
}
