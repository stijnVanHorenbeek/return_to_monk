use std::io::{stdin, stdout, Write};

use crate::lexer::Lexer;
use crate::token::Token;

pub fn start_repl() {
    println!("Return to Monk REPL (Ctrl+C to exit)");
    loop {
        print!(">> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let mut lexer = Lexer::new(&input);
                loop {
                    let tok = lexer.next_token();
                    if tok == Token::EOF {
                        break;
                    }
                    println!("{}", tok);
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
