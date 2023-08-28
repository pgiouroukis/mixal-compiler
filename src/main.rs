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
            var i, j: int; \
            i = 1; \
            j = 2; \
            while (i == 2) { \
                if (i == 2) { \
                    continue; \
                } \
                break; \
            } \
            if (i == 1) {
                break; \
            }
            if (i == 2) {
                continue; \
            }            
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
