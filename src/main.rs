mod parser;
mod lexer;
mod utilities;
mod mixal;
mod semantic_analyzer;

use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler};
use crate::parser::Parser;
use crate::semantic_analyzer::SemanticAnalyzer;

fn main() {

    let program: String = String::from(
        "{ \
            var x, y: int; \
            x = 5; \
            y = 1 + 4 * 3 - 13; \
            print x / y; \
        }",
    );

    let tokens = get_tokens_from_program(&program);
    let mut parser = Parser::new(tokens);
    if parser.analyze_grammar() {
        println!("Parsing successful");
    } else {
        println!("Parsing failed");
        return;
    }

    let mut semantic_checker = SemanticAnalyzer::new(&parser.ast);
    if semantic_checker.run() {
        println!("All semantic checks passed");
    } else {
        println!("Some semantic checks failed");
        return;
    }

    let mut assembler = MixalAssembler::new(
        parser.ast.clone(), 
        String::from("program.mixal")
    );
    assembler.run();
}
