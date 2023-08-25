mod parser;
mod lexer;
mod utilities;
mod mixal;

use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler};
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            var alpha, beta, gamma, k, l: int; \
            for (alpha = 1; alpha < 10; alpha += 1) { \
                if (alpha == 3) gamma = 32; \
                if (alpha % 2) { \
                    beta += 1; \
                } \
                for (k = 1; k == 1; k += 0) \
                    break; \
                for (k = 1; k < 10; k += 1) { \
                    if (k % 2) \
                        continue; \
                    l += 1; \
                } \
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
