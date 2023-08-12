mod parser;
mod lexer;
mod utilities;

use crate::utilities::get_tokens_from_program;
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            var i: int; \
            for (i = 1; i < 10; i+=1) { \
                for (kk = 50; kk != 10; kk-=5) \
                    if (a == 4) print 4; \
                break; \
            } \
        }",
    );

    let tokens = get_tokens_from_program(&program);
    let mut parser = Parser::new(tokens);
    
    if parser.analyze_grammar() {
        println!("Parsing successful");
        println!("AST: {:?}", parser.ast)
    } else {
        println!("Parsing failed");
    }
}
