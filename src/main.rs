mod parser;
mod lexer;
mod utilities;

use crate::utilities::get_tokens_from_program;
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            a = -1 + (-5); \
        }",
    );

    let tokens = get_tokens_from_program(&program);
    let mut parser = Parser::new(tokens);
    
    if parser.analyze_grammar() {
        println!("Parsing successful");
    } else {
        println!("Parsing failed");
    }
}
