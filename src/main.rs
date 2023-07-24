mod ast;
mod lexer;
mod utilities;

use crate::utilities::get_tokens_from_program;
use crate::ast::Parser;

fn main() {

    let program: String = String::from(
        "{  
            var first, second, third : int; \
            var alpha, beta, gamma : int; \
            first += (second + third / 2) && (alpha || beta); \
            if (alpha + 4) { \
                print a; \
                first += (second + third / 2) && (alpha || beta); \
            } else { \
                print a+5; \
                first += (second + third / 2) && (alpha || beta); \
            }\
        }",
    );

    let tokens = get_tokens_from_program(&program);
    let mut parser = Parser::new(tokens);
    
    if parser.analyze_grammar() {
        println!("Parsing successful")
    } else {
        println!("Parsing failed")
    }
}
