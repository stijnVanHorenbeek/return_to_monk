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
        Expression::Prefix(operator, right) => {
            let right = eval_expression(right)?;
            eval_prefix_expression(&operator.to_string(), right)
        }
        _ => Err(Error::msg("not implemented")),
    }
}

fn eval_prefix_expression(operator: &str, right: Object) -> Result<Object, anyhow::Error> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => Err(Error::msg("unknown operator")),
    }
}

fn eval_bang_operator_expression(right: Object) -> Result<Object, anyhow::Error> {
    match right {
        Object::Boolean(value) => Ok(Object::Boolean(!value)),
        Object::Null => Ok(Object::Boolean(true)),
        _ => Ok(Object::Boolean(false)),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object, anyhow::Error> {
    match right {
        Object::Integer(value) => Ok(Object::Integer(-value)),
        _ => Ok(Object::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![("true", true), ("false", false)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_boolean_object(evaluated, expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_boolean_object(evaluated, expected);
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

    fn test_boolean_object(obj: Object, expected: bool) {
        match obj {
            Object::Boolean(value) => {
                assert_eq!(value, expected);
            }
            _ => {
                panic!("Object is not Boolean. got={}", obj);
            }
        }
    }
}
