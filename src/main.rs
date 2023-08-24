mod parser;
mod lexer;
mod utilities;
mod mixal;

use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler};
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            var alpha, beta: int; \
            if (alpha + 3 * 4) { \
                beta += 4 + 2; \
            } else if (alpha == 0) { \
                beta += 4; \
            } else beta = 3; \
            if (beta == 3) alpha = 3; \
            else beta += 3; \
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

    let mut assembler = MixalAssembler::new(
        parser.ast.clone(), 
        String::from("program.mixal")
    );
    assembler.run();
}
