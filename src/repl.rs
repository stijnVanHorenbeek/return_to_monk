use std::io::{stdin, stdout, Write};

use crate::evaluator::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub fn start_repl() {
    println!("Return to Monk REPL (Ctrl+C to exit)");
    loop {
        print!(">> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let lexer = Lexer::new(&input);
                let mut parser = Parser::new(lexer);
                let program = parser.parse_program();
                if parser.errors.len() > 0 {
                    println!("ya done f'ed up");
                    for error in parser.errors {
                        println!("{}", error);
                    }
                }

                let evaluated = eval(program);
                match evaluated {
                    Ok(obj) => println!("{}", obj),
                    Err(error) => println!("error: {}", error),
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
