use orange_trees::Node;
use std::{fs::File, io::Write};
use std::process::Command;
use std::collections::HashMap;
use crate::lexer::Token;
use super::{instruction::*, mnemonic::*, register::*, utilities::*};

const PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS: u16 = 2000;

pub struct MixalAssembler {
    pub ast: Node<usize, Token>,
    pub output_file_path: String,
    file: File,
    vtable: HashMap<String, u16>,
    next_memory_address_to_allocate: u16    
}

impl MixalAssembler {
    pub fn new(ast: Node<usize, Token>, output_file_path: String) -> MixalAssembler{
        MixalAssembler {
            ast,
            output_file_path: output_file_path.clone(),
            file: File::create(output_file_path).expect("to be created"),
            vtable: HashMap::new(),            
            // we purposely start this from 1 to save address 0 for 'temp',
            // as some operations may need to allocate to memory temporarily
            next_memory_address_to_allocate: 1            
        }
    }

    pub fn run(&mut self) {
        self.instruction_set_instructions_allocation_address(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        self.handle_root(self.ast.clone());        
        self.instruction_end_program(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        Command::new("mixasm").arg(self.output_file_path.clone()).output().expect("to execute");
    }

    fn handle_root(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        for child in children {
            match child.value() {
                Token::Ast(_) => {
                    self.handle_root(child.clone());
                },
                Token::Int => {
                    self.handle_variable_declaration(child.clone());
                },
                Token::Assignment => {
                    self.handle_assignment_operator(child.clone());
                },
                _ => {}
            }
        }
    }

    fn handle_variable_declaration(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        for child in children {
            let memory_address_to_allocate = self.next_memory_address_to_allocate;
            self.instruction_store_zero_to_address(memory_address_to_allocate);
            if let Token::Id(identifier) = child.value() {
                self.vtable.insert(identifier.clone(), memory_address_to_allocate);
            }
            self.next_memory_address_to_allocate += 1;
        }
    }

    fn handle_assignment_operator(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        let expression_node = children.get(1).expect("to exist");
        
        // We assume this will compute and store 
        // the expression result to register RA
        // TODO: implement 'self.handle_expression_node(expression_node)'
        
        let identifier_token = children.get(0).expect("to exist").value();
        if let Token::Id(identifier) = identifier_token {
            let identifier_memory_address = self.vtable.get(identifier).expect("to exist").clone();
            self.instruction_store_register_to_address(
                identifier_memory_address,
                MixalRegister::RA
            );
        }
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

    fn instruction_store_register_to_address(&mut self, address: u16, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_store_mnemonic(register),
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_store_zero_to_address(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None,
            MixalMnemonic::STZ,
            Some(String::from(format!("{}(0:5)", address)))
        );
        self.write_to_file(instruction.to_string());
    }
}
