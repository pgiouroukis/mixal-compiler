mod parser;
mod lexer;
mod utilities;
mod mixal;
mod files_handler;
mod semantic_analyzer;

use crate::mixal::utilities::run_mix_binary_file_and_print_output;
use crate::{utilities::get_tokens_from_program, mixal::assembler::MixalAssembler, files_handler::FilesHandler};
use crate::parser::Parser;
use crate::semantic_analyzer::SemanticAnalyzer;
use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a YAL source code file path as an argument.");
        return;
    }
    let file_handler = FilesHandler::new(&args[1]);

    println!("------------------------------------");

    let tokens = get_tokens_from_program(&file_handler.yal_source_code);
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
        file_handler.mixal_output_file_path.clone()
    );
    assembler.run();

    println!("Created the MIX executable file at {}", file_handler.mix_output_file_path);

    println!("------------------------------------");

    if args.len() > 2 && args[2] == "--run" {
        run_mix_binary_file_and_print_output(&file_handler.mix_output_file_path);
        println!("------------------------------------");
    }
    
}
