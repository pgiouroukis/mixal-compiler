mod parser;
mod lexer;
mod utilities;
mod mixal;

use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler};
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            var alpha, beta, gamma: int; \
            while (alpha < 10) { \
                while (1) { \
                    if (142 == (140 + 2)*2/2) { \
                        gamma = 42; \
                    } \
                    break; \
                } \
                if (alpha == 2) \
                    beta = 42; \
                alpha += 1; \
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

    let mut assembler = MixalAssembler::new(
        parser.ast.clone(), 
        String::from("program.mixal")
    );
    assembler.run();
}
