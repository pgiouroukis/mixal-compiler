mod parser;
mod lexer;
mod utilities;
mod mixal;

use crate::utilities::get_tokens_from_program;
use crate::parser::Parser;
use mixal::{instruction::MixalInstruction, mnemonic::MixalMnemonic};

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
    
    let mut instruction = MixalInstruction::new(
        None,
        MixalMnemonic::ORIG,
        Some(String::from("2000"))
    );
    println!("{}", instruction.to_string());

    let mut instruction = MixalInstruction::new(
        Some(String::from("START")),
        MixalMnemonic::NOP,
        None
    );
    println!("{}", instruction.to_string());

    let mut instruction = MixalInstruction::new(
        None,
        MixalMnemonic::HLT,
        None
    );
    println!("{}", instruction.to_string());

    let mut instruction = MixalInstruction::new(
        None,
        MixalMnemonic::END,
        Some(String::from("START"))
    );
    println!("{}", instruction.to_string());    
}
