use crate::ast::*;
use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display, rc::Rc};

#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(isize),
    Boolean(bool),
    ReturnValue(Rc<Object>),
    Function(Function),
    Null,
}

impl Object {
    fn is_truthy(&self) -> bool {
        match self {
            Object::Null => false,
            Object::Boolean(value) => *value,
            _ => true,
        }
    }

    fn type_of(&self) -> &str {
        match self {
            Object::Integer(_) => "INTEGER",
            Object::Boolean(_) => "BOOLEAN",
            Object::ReturnValue(value) => value.type_of(),
            Object::Function(_) => "FUNCTION",
            Object::Null => "NULL",
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::ReturnValue(value) => write!(f, "{}", value),
            Object::Function(value) => write!(f, "{}", value),
            Object::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    parameters: Vec<String>,
    body: Statement,
    env: Environment,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str("fn(");
        for (i, param) in self.parameters.iter().enumerate() {
            result.push_str(param);
            if i != self.parameters.len() - 1 {
                result.push_str(", ");
            }
        }
        result.push_str(") {\n");
        result.push_str(&format!("{}", self.body));
        result.push_str("\n}");
        write!(f, "{}", result)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Rc<Object>>,
    outer: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<Environment>) -> Environment {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<Object>> {
        match self.store.get(name) {
            Some(value) => Some(value.clone()),
            None => match &self.outer {
                Some(outer) => outer.get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: &str, value: Rc<Object>) {
        self.store.insert(name.to_string(), value);
    }
}

pub fn eval(program: Program, env: &mut Environment) -> Result<Rc<Object>> {
    eval_statements(&program.statements, env)
}

fn eval_statements(
    statements: &[Statement],
    env: &mut Environment,
) -> Result<Rc<Object>, anyhow::Error> {
    let mut result = Rc::new(Object::Null);

    for statement in statements {
        result = eval_statement(statement, env)?;

        match &*result {
            Object::ReturnValue(obj) => return Ok(obj.clone()),
            _ => {}
        }
    }

    Ok(result)
}

fn eval_block_statement(
    statements: &[Statement],
    env: &mut Environment,
) -> Result<Rc<Object>, anyhow::Error> {
    let mut result = Rc::new(Object::Null);

    for statement in statements {
        result = eval_statement(statement, env)?;

        match &*result {
            Object::ReturnValue(_) => return Ok(result.into()),
            _ => {}
        }
    }

    Ok(result.into())
}

fn eval_statement(
    statement: &Statement,
    env: &mut Environment,
) -> Result<Rc<Object>, anyhow::Error> {
    match statement {
        Statement::LetStatement { name, value } => {
            let obj = eval_expression(value, env)?;
            env.set(name, obj.clone());
            Ok(obj)
        }
        Statement::ExpressionStatement(expression) => eval_expression(expression, env),
        Statement::BlockStatement(statements) => eval_block_statement(statements, env),
        Statement::ReturnStatement(value) => {
            let obj = eval_expression(value, env)?;
            Ok(Object::ReturnValue(obj).into())
        }
    }
}

fn eval_expression(
    expression: &Expression,
    env: &mut Environment,
) -> Result<Rc<Object>, anyhow::Error> {
    match expression {
        Expression::IntegerLiteral(value) => Ok(Rc::new(Object::Integer(*value))),
        Expression::BooleanLiteral(value) => Ok(Rc::new(Object::Boolean(*value))),
        Expression::Prefix(operator, right) => {
            let right = eval_expression(right, env)?;
            eval_prefix_expression(&operator, right)
        }
        Expression::Infix(operator, left, right) => {
            let left = eval_expression(&left, env)?;
            let right = eval_expression(&right, env)?;
            eval_infix_expression(&operator, &left, &right)
        }
        Expression::If {
            condition,
            consequence,
            alternative,
        } => {
            let condition = eval_expression(&condition, env)?;
            if condition.is_truthy() {
                eval_statement(&consequence, env)
            } else if let Some(alternative) = alternative {
                eval_statement(&alternative, env)
            } else {
                Ok(Object::Null.into())
            }
        }
        Expression::Identifier(name) => match env.get(name) {
            Some(value) => Ok(value),
            None => Err(anyhow!("identifier not found: {}", name)),
        },
        Expression::FunctionLiteral { parameters, body } => {
            // TODO: Clone is not efficient
            let func = Function {
                parameters: parameters.clone(),
                body: *body.clone(),
                env: env.clone(),
            };
            Ok(Object::Function(func).into())
        }
        Expression::Call {
            function,
            arguments,
        } => {
            let func = eval_expression(function, env)?;
            let args = eval_expressions(arguments, env)?;
            apply_function(func, args)
        }
    }
}

fn eval_expressions(
    expressions: &[Expression],
    env: &mut Environment,
) -> Result<Vec<Rc<Object>>, anyhow::Error> {
    let mut result = Vec::new();

    for expression in expressions {
        let evaluated = eval_expression(expression, env)?;
        result.push(evaluated);
    }

    Ok(result)
}

fn apply_function(func: Rc<Object>, args: Vec<Rc<Object>>) -> Result<Rc<Object>, anyhow::Error> {
    match &*func {
        Object::Function(function) => {
            let mut extended_env = Environment::new_enclosed(function.env.clone().into());

            for (param, arg) in function.parameters.iter().zip(args) {
                extended_env.set(param, arg);
            }

            let evaluated = eval_statement(&function.body, &mut extended_env)?;
            match &*evaluated {
                Object::ReturnValue(value) => Ok(value.clone()),
                _ => Ok(evaluated),
            }
        }
        _ => Err(anyhow!("not a function: {}", func)),
    }
}

fn eval_prefix_expression(
    operator: &Prefix,
    right: Rc<Object>,
) -> Result<Rc<Object>, anyhow::Error> {
    match operator {
        Prefix::BANG => eval_bang_operator_expression(right),
        Prefix::MINUS => eval_minus_prefix_operator_expression(right),
    }
}

fn eval_bang_operator_expression(right: Rc<Object>) -> Result<Rc<Object>, anyhow::Error> {
    match &*right {
        Object::Boolean(value) => Ok(Object::Boolean(!value).into()),
        Object::Null => Ok(Object::Boolean(true).into()),
        _ => Ok(Object::Boolean(false).into()),
    }
}

fn eval_minus_prefix_operator_expression(right: Rc<Object>) -> Result<Rc<Object>, anyhow::Error> {
    match &*right {
        Object::Integer(value) => Ok(Object::Integer(-value).into()),
        _ => Err(anyhow!("unknown operator: -{}", right.type_of())),
    }
}

fn eval_infix_expression(
    operator: &Infix,
    left: &Object,
    right: &Object,
) -> Result<Rc<Object>, anyhow::Error> {
    if left.type_of() != right.type_of() {
        return Err(anyhow!(
            "type mismatch: {} {} {}",
            left.type_of(),
            operator,
            right.type_of()
        ));
    }

    match (operator, left, right) {
        (_, Object::Integer(left), Object::Integer(right)) => {
            eval_integer_infix_expression(operator, *left, *right)
        }
        (Infix::EQ, left, right) => Ok(Object::Boolean(left == right).into()),
        (Infix::NOT_EQ, left, right) => Ok(Object::Boolean(left != right).into()),
        _ => Err(anyhow!(
            "unknown operator: {} {} {}",
            left.type_of(),
            operator,
            right.type_of()
        )),
    }
}

fn eval_integer_infix_expression(
    operator: &Infix,
    left: isize,
    right: isize,
) -> Result<Rc<Object>, anyhow::Error> {
    match operator {
        Infix::PLUS => Ok(Object::Integer(left + right).into()),
        Infix::MINUS => Ok(Object::Integer(left - right).into()),
        Infix::ASTERISK => Ok(Object::Integer(left * right).into()),
        Infix::SLASH => Ok(Object::Integer(left / right).into()),
        Infix::LT => Ok(Object::Boolean(left < right).into()),
        Infix::GT => Ok(Object::Boolean(left > right).into()),
        Infix::EQ => Ok(Object::Boolean(left == right).into()),
        Infix::NOT_EQ => Ok(Object::Boolean(left != right).into()),
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
            let evaluated = test_eval(input).unwrap();
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
            let evaluated = test_eval(input).unwrap();
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
            let evaluated = test_eval(input).unwrap();
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
            let evaluated = test_eval(input).unwrap();
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
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_function_object() {
        let tests = vec![("fn(x) { x + 2; };", vec!["x"], "fn(x) {\n(x + 2)\n}")];

        for (input, expected_params, expected_func) in tests {
            let evaluated = test_eval(input).unwrap();

            match &*evaluated {
                Object::Function(function) => {
                    assert_eq!(function.parameters, expected_params);
                    assert_eq!(format!("{}", function), expected_func);
                }
                _ => {
                    panic!("object is not Function. got={}", evaluated);
                }
            }
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", 5),
            ("let identity = fn(x) { return x; }; identity(5);", 5),
            ("let double = fn(x) { x * 2; }; double(5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
            ("fn(x) { x; }(5)", 5),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input).unwrap();
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_closures() {
        let input = r#"
        let newAdder = fn(x) {
            fn(y) { x + y };
        };
        let addTwo = newAdder(2);
        addTwo(3);
        "#;
        let expected = 5;

        let evaluated = test_eval(input).unwrap();
        test_integer_object(evaluated, expected);
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
            ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
            ("-true", "unknown operator: -BOOLEAN"),
            ("true + false;", "unknown operator: BOOLEAN + BOOLEAN"),
            ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                r#"
                if (10 > 1) {
                    if (10 > 1) {
                        return true + false;
                    }
                    return 1;
                }
                "#,
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            ("foobar", "identifier not found: foobar"),
        ];

        for (input, expected_message) in tests {
            let evaluated = test_eval(input);

            match evaluated {
                Err(err) => {
                    assert_eq!(err.to_string(), expected_message);
                }
                Ok(_) => {
                    panic!("no error object returned. got={}", evaluated.unwrap());
                }
            }
        }
    }

    fn test_eval(input: &str) -> Result<Rc<Object>, anyhow::Error> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        let mut env = Environment::new();
        eval(program, &mut env)
    }

    fn test_integer_object(obj: Rc<Object>, expected: isize) {
        match *obj {
            Object::Integer(value) => {
                assert_eq!(value, expected);
            }
            _ => test_null_object(obj),
        }
    }

    fn test_boolean_object(obj: Rc<Object>, expected: bool) {
        match *obj {
            Object::Boolean(value) => {
                assert_eq!(value, expected);
            }
            _ => {
                panic!("Object is not Boolean. got={}", obj);
            }
        }
    }

    fn test_null_object(obj: Rc<Object>) {
        match *obj {
            Object::Null => {}
            _ => {
                panic!("Object is not Null. got={}", obj);
            }
        }
    }
}
