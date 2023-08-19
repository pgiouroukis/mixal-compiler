use orange_trees::Node;
use std::{fs::File, io::Write};
use std::process::Command;
use crate::lexer::Token;
use super::{instruction::*, mnemonic::*};

const PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS: u16 = 2000;

pub struct MixalAssembler {
    pub ast: Node<usize, Token>,
    pub output_file_path: String,
    file: File,
}

impl MixalAssembler {
    pub fn new(ast: Node<usize, Token>, output_file_path: String) -> MixalAssembler{
        MixalAssembler {
            ast,
            output_file_path: output_file_path.clone(),
            file: File::create(output_file_path).expect("to be created"),
        }
    }

    pub fn run(&mut self) {
        self.instruction_set_instructions_allocation_address(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        self.instruction_end_program(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        Command::new("mixasm").arg(self.output_file_path.clone()).output().expect("to execute");
    }

    fn write_to_file(&mut self, str: String) {
        self.file.write_all(str.as_bytes()).expect("to be written");
    }

    // ---------------------------------------------
    //             MIXAL INSTRUCTIONS              
    //  The methods below model MIXAL instructions.
    //  They are prefixed with 'instruction_'.
    //  When invoked, they will append the 
    //  instruction to 'self.output_file_path'
    // ---------------------------------------------

    fn instruction_set_instructions_allocation_address(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None,
            MixalMnemonic::ORIG,
            Some(String::from(address.to_string()))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_end_program(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None,
            MixalMnemonic::END,
            Some(String::from(address.to_string()))
        );
        self.write_to_file(instruction.to_string());
    }
}