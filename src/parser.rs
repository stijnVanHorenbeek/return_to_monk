#![allow(dead_code)]

use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

type PrefixParseFn = fn(p: &mut Parser) -> Option<Expression>;
type InfixParseFn = fn(p: &mut Parser, e: Expression) -> Option<Expression>;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    LOWEST = 0,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::EOF,
            peek_token: Token::EOF,
            errors: Vec::new(),
        };

        parser.next_token();
        parser.next_token();
        parser
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program::new();

        while self.current_token != Token::EOF {
            let statement = self.parse_statement();
            if let Some(statement) = statement {
                program.statements.push(statement);
            }
            self.next_token();
        }
        program
    }

    // need to inline this to avoid borrow checker issues
    #[inline]
    fn prefix_parse_fns(token: &Token) -> Option<PrefixParseFn> {
        match token {
            Token::IDENT(_) => Some(Parser::parse_identifier),
            Token::INT(_) => Some(Parser::parse_integer_literal),
            Token::LPAREN => Some(Parser::parse_grouped_expression),
            Token::IF => Some(Parser::parse_if_expression),
            Token::FUNCTION => Some(Parser::parse_function_literal),
            Token::TRUE | Token::FALSE => Some(Parser::parse_boolean_literal),
            Token::BANG | Token::MINUS => Some(Parser::parse_prefix),
            _ => None,
        }
    }

    // need to inline this to avoid borrow checker issues
    #[inline]
    fn infix_parse_fns(token: &Token) -> Option<InfixParseFn> {
        match token {
            Token::PLUS
            | Token::MINUS
            | Token::ASTERISK
            | Token::SLASH
            | Token::EQ
            | Token::NOT_EQ
            | Token::LT
            | Token::GT => Some(Parser::parse_infix),
            Token::LPAREN => Some(Parser::parse_call_expression),
            _ => None,
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current_token {
            Token::LET => self.parse_let_statement(),
            Token::RETURN => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let name = match &self.peek_token {
            Token::IDENT(s) => s.clone(),
            _ => return None,
        };
        self.next_token();
        if !self.expect_peek(&Token::ASSIGN) {
            return None;
        }
        while !self.current_token_is(&Token::SEMICOLON) {
            self.next_token();
        }
        Some(Statement::LetStatement { name, value: None })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();
        while !self.current_token_is(&Token::SEMICOLON) {
            self.next_token();
        }
        Some(Statement::ReturnStatement(None))
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
        self.next_token();
        let mut statements = Vec::new();

        while !self.current_token_is(&Token::RBRACE) && !self.current_token_is(&Token::EOF) {
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            }
            self.next_token();
        }
        Some(Statement::BlockStatement(statements))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expression = match self.parse_expression(Precedence::LOWEST) {
            Some(expression) => expression,
            None => return None,
        };

        let statement = Statement::ExpressionStatement(expression);

        if self.peek_token_is(&Token::SEMICOLON) {
            self.next_token();
        }

        Some(statement)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left = match Parser::prefix_parse_fns(&self.current_token) {
            Some(prefix) => prefix(self),
            None => {
                self.errors.push(format!(
                    "no prefix parse function for {:?}",
                    self.current_token
                ));
                return None;
            }
        };

        // check if there are any infix parse functions for the current token
        // and if the precedence of the infix parse function is greater than the
        // current precedence
        while !self.peek_token_is(&Token::SEMICOLON) {
            if precedence as i32 >= self.peek_precedence() as i32 {
                break;
            }
            let infix = match Parser::infix_parse_fns(&self.peek_token) {
                Some(infix) => infix,
                None => return left,
            };

            self.next_token();

            left = match left {
                Some(left) => infix(self, left),
                None => return None,
            };
        }

        left
    }

    fn parse_identifier(p: &mut Parser) -> Option<Expression> {
        match &p.current_token {
            Token::IDENT(s) => Some(Expression::Identifier(s.into())),
            _ => None,
        }
    }

    fn parse_integer_literal(p: &mut Parser) -> Option<Expression> {
        match &p.current_token {
            Token::INT(i) => Some(Expression::IntegerLiteral(i.clone())),
            _ => None,
        }
    }

    fn parse_boolean_literal(p: &mut Parser) -> Option<Expression> {
        match &p.current_token {
            Token::TRUE => Some(Expression::BooleanLiteral(true)),
            Token::FALSE => Some(Expression::BooleanLiteral(false)),
            _ => None,
        }
    }

    fn parse_grouped_expression(p: &mut Parser) -> Option<Expression> {
        p.next_token();

        let expression = p.parse_expression(Precedence::LOWEST);

        if !p.expect_peek(&Token::RPAREN) {
            return None;
        }
        expression
    }

    fn parse_if_expression(p: &mut Parser) -> Option<Expression> {
        if !p.expect_peek(&Token::LPAREN) {
            return None;
        }

        p.next_token();
        let condition = match p.parse_expression(Precedence::LOWEST) {
            Some(condition) => condition,
            None => return None,
        };

        if !p.expect_peek(&Token::RPAREN) {
            return None;
        }

        if !p.expect_peek(&Token::LBRACE) {
            return None;
        }

        let consequence = match p.parse_block_statement() {
            Some(consequence) => consequence,
            None => return None,
        };

        let alternative = if p.peek_token_is(&Token::ELSE) {
            p.next_token();
            if !p.expect_peek(&Token::LBRACE) {
                return None;
            }
            match p.parse_block_statement() {
                Some(alternative) => Some(alternative),
                None => return None,
            }
        } else {
            None
        };

        Some(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: alternative.map(Box::new),
        })
    }

    fn parse_function_literal(p: &mut Parser) -> Option<Expression> {
        if !p.expect_peek(&Token::LPAREN) {
            return None;
        }

        let parameters = match p.parse_function_parameters() {
            Some(parameters) => parameters,
            _ => return None,
        };

        if !p.expect_peek(&Token::LBRACE) {
            return None;
        }

        let body = match p.parse_block_statement() {
            Some(body) => body,
            _ => return None,
        };

        Some(Expression::FunctionLiteral {
            parameters,
            body: Box::new(body),
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<String>> {
        let mut identifiers = vec![];

        // no function parameters
        if self.peek_token_is(&Token::RPAREN) {
            self.next_token();
            return Some(identifiers);
        }

        self.next_token();
        // first parameter
        identifiers.push(match &self.current_token {
            Token::IDENT(s) => s.clone(),
            _ => return None,
        });

        // optional additional parameters
        while self.peek_token_is(&Token::COMMA) {
            self.next_token();
            self.next_token();
            identifiers.push(match &self.current_token {
                Token::IDENT(s) => s.clone(),
                _ => return None,
            });
        }

        if !self.expect_peek(&Token::RPAREN) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_prefix(p: &mut Parser) -> Option<Expression> {
        let operator = match &p.current_token {
            Token::BANG => Prefix::BANG,
            Token::MINUS => Prefix::MINUS,
            _ => return None,
        };

        p.next_token();

        let right = match p.parse_expression(Precedence::PREFIX) {
            Some(right) => right,
            None => return None,
        };

        Some(Expression::Prefix(operator, Box::new(right)))
    }

    fn parse_infix(p: &mut Parser, left: Expression) -> Option<Expression> {
        let operator = match &p.current_token {
            Token::PLUS => Infix::PLUS,
            Token::MINUS => Infix::MINUS,
            Token::ASTERISK => Infix::ASTERISK,
            Token::SLASH => Infix::SLASH,
            Token::EQ => Infix::EQ,
            Token::NOT_EQ => Infix::NOT_EQ,
            Token::LT => Infix::LT,
            Token::GT => Infix::GT,
            _ => return None,
        };

        let precedence = p.current_precedence();
        p.next_token();

        if let Some(right) = p.parse_expression(precedence) {
            Some(Expression::Infix(operator, Box::new(left), Box::new(right)))
        } else {
            None
        }
    }

    fn current_precedence(&self) -> Precedence {
        match &self.current_token {
            Token::EQ | Token::NOT_EQ => Precedence::EQUALS,
            Token::LT | Token::GT => Precedence::LESSGREATER,
            Token::PLUS | Token::MINUS => Precedence::SUM,
            Token::ASTERISK | Token::SLASH => Precedence::PRODUCT,
            Token::LPAREN => Precedence::CALL,
            _ => Precedence::LOWEST,
        }
    }

    fn peek_precedence(&self) -> Precedence {
        match &self.peek_token {
            Token::EQ | Token::NOT_EQ => Precedence::EQUALS,
            Token::LT | Token::GT => Precedence::LESSGREATER,
            Token::PLUS | Token::MINUS => Precedence::SUM,
            Token::ASTERISK | Token::SLASH => Precedence::PRODUCT,
            Token::LPAREN => Precedence::CALL,
            _ => Precedence::LOWEST,
        }
    }

    fn parse_call_expression(p: &mut Parser, function: Expression) -> Option<Expression> {
        let arguments = match p.parse_call_arguments() {
            Some(arguments) => arguments,
            _ => return None,
        };

        Some(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
        let mut arguments = vec![];

        // no arguments
        if self.peek_token_is(&Token::RPAREN) {
            self.next_token();
            return Some(arguments);
        }

        // first argument
        self.next_token();
        if let Some(argument) = self.parse_expression(Precedence::LOWEST) {
            arguments.push(argument);
        } else {
            return None;
        }

        // optional additional arguments
        while self.peek_token_is(&Token::COMMA) {
            self.next_token();
            self.next_token();
            if let Some(argument) = self.parse_expression(Precedence::LOWEST) {
                arguments.push(argument);
            } else {
                return None;
            }
        }

        if !self.expect_peek(&Token::RPAREN) {
            return None;
        }

        Some(arguments)
    }

    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    fn peek_token_is(&self, token: &Token) -> bool {
        self.peek_token == *token
    }

    fn peek_error(&mut self, token: &Token) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            token, self.peek_token
        );
        self.errors.push(msg);
    }

    fn expect_peek(&mut self, token: &Token) -> bool {
        if self.peek_token_is(token) {
            self.next_token();
            true
        } else {
            self.peek_error(token);
            false
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_statements() {
        let input = r#"
let x = 5;
let y = 10;
let foobar = 838383;
"#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(parser.errors.len(), 0);

        let tests = vec!["x", "y", "foobar"];

        for (i, test) in tests.iter().enumerate() {
            let statement = &program.statements[i];
            match statement {
                Statement::LetStatement { name, value: _ } => {
                    assert_eq!(name, *test);
                }
                _ => panic!("Expected LetStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = r#"return 5;
return 10;
return 993322;"#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ReturnStatement(_) => {}
                _ => panic!("Expected ReturnStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ExpressionStatement(Expression::Identifier(s)) => {
                    assert_eq!(s, "foobar");
                }
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ExpressionStatement(Expression::IntegerLiteral(i)) => {
                    assert_eq!(i, 5);
                }
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_boolean_literal_expression() {
        let tests = vec![("true;", true), ("false;", false)];

        for test in tests {
            let (input, value) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in program.statements {
                match statement {
                    Statement::ExpressionStatement(Expression::BooleanLiteral(b)) => {
                        assert_eq!(b, value);
                    }
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        // assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ExpressionStatement(Expression::If {
                    condition,
                    consequence,
                    alternative,
                }) => {
                    assert_eq!(condition.to_string(), "(x < y)");
                    assert_eq!(consequence.to_string(), "x");
                    assert_eq!(alternative, None);
                }
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ExpressionStatement(Expression::If {
                    condition,
                    consequence,
                    alternative,
                }) => {
                    assert_eq!(condition.to_string(), "(x < y)");
                    assert_eq!(consequence.to_string(), "x");
                    assert_eq!(alternative.unwrap().to_string(), "y");
                }
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in program.statements {
            match statement {
                Statement::ExpressionStatement(Expression::FunctionLiteral {
                    parameters,
                    body,
                }) => {
                    assert_eq!(parameters.len(), 2);
                    assert_eq!(parameters[0], "x");
                    assert_eq!(parameters[1], "y");
                    assert_eq!(body.to_string(), "(x + y)");
                }
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_function_parameter_parsing() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec!["x"]),
            ("fn(x, y, z) {};", vec!["x", "y", "z"]),
        ];

        for test in tests {
            let (input, expected) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in program.statements {
                match statement {
                    Statement::ExpressionStatement(Expression::FunctionLiteral {
                        parameters,
                        body: _,
                    }) => {
                        assert_eq!(parameters, expected);
                    }
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_integer_prefix_expression() {
        let tests = vec![("!5", "!", 5), ("-15", "-", 15)];

        for test in tests {
            let (input, prefix, int) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in &program.statements {
                println!("{:?}", statement);
                match statement {
                    Statement::ExpressionStatement(exp) => match exp {
                        Expression::Prefix(p, r) => {
                            assert_eq!(p.to_string(), prefix);
                            assert_eq!(is_literal_expression(r, &int.to_string()), true);
                        }
                        _ => panic!("Expected PrefixExpression, got {:?}", exp),
                    },
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_boolean_prefix_expression() {
        let tests = vec![("!true", "!", "true"), ("!false", "!", "false")];

        for test in tests {
            let (input, prefix, value) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in &program.statements {
                match statement {
                    Statement::ExpressionStatement(exp) => match exp {
                        Expression::Prefix(p, r) => {
                            assert_eq!(p.to_string(), prefix);
                            assert_eq!(is_literal_expression(r, value), true);
                        }
                        _ => panic!("Expected PrefixExpression, got {:?}", exp),
                    },
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 1);
        assert_eq!(parser.errors.len(), 0);

        for statement in &program.statements {
            match statement {
                Statement::ExpressionStatement(exp) => match exp {
                    Expression::Call {
                        function,
                        arguments,
                    } => {
                        assert_eq!(function.to_string(), "add");
                        assert_eq!(arguments.len(), 3);
                        assert_eq!(arguments[0].to_string(), "1");
                        assert_eq!(arguments[1].to_string(), "(2 * 3)");
                        assert_eq!(arguments[2].to_string(), "(4 + 5)");
                    }
                    _ => panic!("Expected CallExpression, got {:?}", exp),
                },
                _ => panic!("Expected ExpressionStatement, got {:?}", statement),
            }
        }
    }

    #[test]
    fn test_integer_infix_expression() {
        let tests = vec![
            ("5 + 4;", 5, "+", 4),
            ("5 - 4;", 5, "-", 4),
            ("5 * 5;", 5, "*", 5),
            ("5 / 5;", 5, "/", 5),
            ("5 > 5;", 5, ">", 5),
            ("5 < 5;", 5, "<", 5),
            ("5 == 5;", 5, "==", 5),
            ("5 != 5;", 5, "!=", 5),
        ];

        for test in tests {
            let (input, left, op, right) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in &program.statements {
                match statement {
                    Statement::ExpressionStatement(exp) => {
                        assert_eq!(
                            is_infix_expression(exp, &left.to_string(), op, &right.to_string()),
                            true
                        );
                    }
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_boolean_infix_expression() {
        let tests = vec![
            ("true == true", true, "==", true),
            ("true != false", true, "!=", false),
            ("false == false", false, "==", false),
        ];

        for test in tests {
            let (input, left, op, right) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(program.statements.len(), 1);
            assert_eq!(parser.errors.len(), 0);

            for statement in &program.statements {
                match statement {
                    Statement::ExpressionStatement(exp) => {
                        assert_eq!(
                            is_infix_expression(exp, &left.to_string(), op, &right.to_string()),
                            true
                        );
                    }
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true", "true"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
            ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];

        for test in tests {
            let (input, expected) = test;
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();

            assert_eq!(parser.errors.len(), 0);
            assert_eq!(program.to_string(), expected);
        }
    }

    fn is_integer_literal(exp: &Expression, value: isize) -> bool {
        match exp {
            Expression::IntegerLiteral(i) => *i == value,
            _ => false,
        }
    }

    fn is_identifier(exp: &Expression, value: &str) -> bool {
        match exp {
            Expression::Identifier(s) => s == value,
            _ => false,
        }
    }

    fn is_literal_expression(exp: &Expression, expected: &str) -> bool {
        match exp {
            Expression::BooleanLiteral(b) => b.to_string() == expected,
            Expression::IntegerLiteral(_) => is_integer_literal(exp, expected.parse().unwrap()),
            Expression::Identifier(_) => is_identifier(exp, expected),
            _ => false,
        }
    }

    fn is_infix_expression(exp: &Expression, left: &str, op: &str, right: &str) -> bool {
        match exp {
            Expression::Infix(o, l, r) => {
                is_literal_expression(l, left)
                    && is_literal_expression(r, right)
                    && o.to_string() == op
            }
            _ => false,
        }
    }
}
