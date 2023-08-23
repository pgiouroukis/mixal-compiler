mod parser;
mod lexer;
mod utilities;
mod mixal;

use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler};
use crate::parser::Parser;

fn main() {

    let program: String = String::from(
        "{ \
            var alpha: int; \
            alpha = !(5 + 2 > 3 || 5 + 3 * 2 + 1 < 100) && (-(35 - 37) < 0); \
            alpha = 1 + !2; \
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
