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

enum Precedence {
    LOWEST,
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
            Token::BANG | Token::MINUS => Some(Parser::parse_prefix),
            _ => None,
        }
    }

    // need to inline this to avoid borrow checker issues
    #[inline]
    fn infix_parse_fns(token: &Token) -> Option<InfixParseFn> {
        None
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
        let left = match Parser::prefix_parse_fns(&self.current_token) {
            Some(prefix) => prefix(self),
            None => None,
        };
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

    fn is_integer_literal(exp: &Expression, value: isize) -> bool {
        match exp {
            Expression::IntegerLiteral(i) => *i == value,
            _ => false,
        }
    }

    #[test]
    fn test_prefix_expression() {
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
                            assert_eq!(is_integer_literal(r, int), true);
                        }
                        _ => panic!("Expected PrefixExpression, got {:?}", exp),
                    },
                    _ => panic!("Expected ExpressionStatement, got {:?}", statement),
                }
            }
        }
    }
}
