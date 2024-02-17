use crate::ast::*;
use anyhow::{Error, Result};
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(isize),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Null,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::ReturnValue(value) => write!(f, "{}", value),
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

        match result {
            Object::ReturnValue(obj) => return Ok(*obj),
            _ => {}
        }
    }

    Ok(result)
}

fn eval_block_statement(statements: &[Statement]) -> Result<Object, anyhow::Error> {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement)?;

        match result {
            Object::ReturnValue(_) => return Ok(result),
            _ => {}
        }
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
        Statement::BlockStatement(statements) => eval_block_statement(statements),
        Statement::ReturnStatement(value) => {
            let obj = eval_expression(value)?;
            println!("returning: {}", obj);
            Ok(Object::ReturnValue(Box::new(obj)))
        }
    }
}

fn eval_expression(expression: &Expression) -> Result<Object, anyhow::Error> {
    match expression {
        Expression::IntegerLiteral(value) => Ok(Object::Integer(*value)),
        Expression::BooleanLiteral(value) => Ok(Object::Boolean(*value)),
        Expression::Prefix(operator, right) => {
            let right = eval_expression(right)?;
            eval_prefix_expression(&operator, right)
        }
        Expression::Infix(operator, left, right) => {
            let left = eval_expression(&left)?;
            let right = eval_expression(&right)?;
            eval_infix_expression(&operator, &left, &right)
        }
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(&condition)?;
            if is_truthy(&condition) {
                eval_statement(&consequence)
            } else if let Some(alternative) = alternative {
                eval_statement(&alternative)
            } else {
                Ok(Object::Null)
            }
        }
        _ => Err(Error::msg("not implemented")),
    }
}

fn eval_prefix_expression(operator: &Prefix, right: Object) -> Result<Object, anyhow::Error> {
    match operator {
        Prefix::BANG => eval_bang_operator_expression(right),
        Prefix::MINUS => eval_minus_prefix_operator_expression(right),
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

fn eval_infix_expression(
    operator: &Infix,
    left: &Object,
    right: &Object,
) -> Result<Object, anyhow::Error> {
    match (operator, left, right) {
        (_, Object::Integer(left), Object::Integer(right)) => {
            eval_integer_infix_expression(operator, *left, *right)
        }
        (Infix::EQ, left, right) => Ok(Object::Boolean(left == right)),
        (Infix::NOT_EQ, left, right) => Ok(Object::Boolean(left != right)),
        _ => Ok(Object::Null),
    }
}

fn eval_integer_infix_expression(
    operator: &Infix,
    left: isize,
    right: isize,
) -> Result<Object, anyhow::Error> {
    match operator {
        Infix::PLUS => Ok(Object::Integer(left + right)),
        Infix::MINUS => Ok(Object::Integer(left - right)),
        Infix::ASTERISK => Ok(Object::Integer(left * right)),
        Infix::SLASH => Ok(Object::Integer(left / right)),
        Infix::LT => Ok(Object::Boolean(left < right)),
        Infix::GT => Ok(Object::Boolean(left > right)),
        Infix::EQ => Ok(Object::Boolean(left == right)),
        Infix::NOT_EQ => Ok(Object::Boolean(left != right)),
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(value) => *value,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

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

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", 10),
            ("if (false) { 10 }", 0),
            ("if (1) { 10 }", 10),
            ("if (1 < 2) { 10 }", 10),
            ("if (1 > 2) { 10 } else { 20 }", 20),
            ("if (1 < 2) { 10 } else { 20 }", 10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            (
                "if (10 > 1) {
                    if (10 > 1) {
                        return 10;
                    }
                    return 1;
                }",
                10,
            ),
        ];

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
            _ => test_null_object(obj),
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

    fn test_null_object(obj: Object) {
        match obj {
            Object::Null => {}
            _ => {
                panic!("Object is not Null. got={}", obj);
            }
        }
    }
}
