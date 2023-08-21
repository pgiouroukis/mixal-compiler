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
        self.handle_expression_node(expression_node.clone());
        
        let identifier_token = children.get(0).expect("to exist").value();
        if let Token::Id(identifier) = identifier_token {
            let identifier_memory_address = self.vtable.get(identifier).expect("to exist").clone();
            self.instruction_store_register_to_address(
                identifier_memory_address,
                MixalRegister::RA
            );
        }
    }

    fn handle_expression_node(&mut self, node: Node<usize, Token>) {
        if let Token::Num(number) = node.value() {
            self.instruction_enter_immediate_value_to_register(*number, MixalRegister::RA);
            return;
        } else if let Token::Id(identifier) = node.value() {
            self.instruction_load_address_to_register(
                self.vtable.get(identifier).expect("to exist").clone(),
                MixalRegister::RA
            );
            return;
        }

        let children = node.children();
        let left_operand = children.get(0).expect("to exist");
        let right_operand = children.get(1).expect("to exist");

        if left_operand.is_leaf() && right_operand.is_leaf() {
            // This is the basic case, where it is possible to directly 
            // compute a result, since the 2 operands are actual values. 
            // Because an operand can be either a Number or a Variable, 
            // we must handle 4 cases, one for every combination. After 
            // computing a result, we store it in register RA so it
            // becomes available for future instructions
            let operator_fn = 
                MixalAssembler::token_to_arithmetic_operator_instruction_fn(node.value());
            match node.value() {
                Token::Plus | Token::Minus | Token::Asterisk => {
                    if let (Token::Num(number1), Token::Num(number2)) = (left_operand.value(), right_operand.value()) {
                        self.instruction_enter_immediate_value_to_register(*number2, MixalRegister::RA);
                        self.instruction_store_register_to_address(0, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(*number1, MixalRegister::RA);
                        operator_fn(self, 0);
                    } else if let (Token::Id(identifier1), Token::Id(identifier2)) = (left_operand.value(), right_operand.value()) {
                        let identifier1_address = self.vtable.get(identifier1).expect("to exist").clone();
                        let identifier2_address = self.vtable.get(identifier2).expect("to exist").clone();
                        self.instruction_load_address_to_register(identifier1_address, MixalRegister::RA);
                        operator_fn(self, identifier2_address);
                    } else if let (Token::Num(number), Token::Id(identifier)) = (left_operand.value(), right_operand.value()) {
                        self.instruction_enter_immediate_value_to_register(number.clone(), MixalRegister::RA);
                        operator_fn(
                            self,
                            self.vtable.get(identifier).expect("to exist").clone()
                        );
                    } else if let (Token::Id(identifier), Token::Num(number)) = (left_operand.value(), right_operand.value()) {
                        let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
                        self.instruction_load_address_to_register(identifier_address, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(*number, MixalRegister::RX);
                        self.instruction_store_register_to_address(0, MixalRegister::RX);
                        operator_fn(self, 0);
                    }
                    if let Token::Asterisk = node.value() {
                        // RA contains the upper bits of the result and 
                        // RX contains the lower bits of the result. 
                        // The sign of the result is stored in the sign bit of RA
                        // For now, we don't handle overflows and only care about
                        // the lower bits of the result
                        self.instruction_store_register_sign_to_address(0, MixalRegister::RA);
                        self.instruction_store_register_without_sign_to_address(0, MixalRegister::RX);
                        self.instruction_load_address_to_register(0, MixalRegister::RA);
                        // TODO: add code that throws exception when the result overflows
                    }                    
                },
                Token::Slash | Token::Percent => {
                    // In the AST, the Token::Num does not store negative numbers.
                    // The only exception to this occurs when an expression uses
                    // the unary minus operator `-x`. In this case, the AST transforms
                    // `-x` to `(-1) * x`, and thus needs to store `Token::Num(-1)`.
                    // But this case is handled in the multiplication operator case above.
                    // So, in the code below, we assume that every Token::Num is positive.
                    // Note that the value of a `Token::Id` in memory CAN be negative.
                    if let (Token::Num(number1), Token::Num(number2)) = (left_operand.value(), right_operand.value()) {
                        self.instruction_enter_immediate_value_to_register(*number2, MixalRegister::RA);
                        self.instruction_store_register_to_address(0, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(0, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(*number1, MixalRegister::RX);
                        operator_fn(self, 0);
                    } if let (Token::Id(identifier1), Token::Id(identifier2)) = (left_operand.value(), right_operand.value()) {
                        let identifier1_address = self.vtable.get(identifier1).expect("to exist").clone();
                        let identifier2_address = self.vtable.get(identifier2).expect("to exist").clone();
                        self.instruction_load_address_to_register(identifier1_address, MixalRegister::RX);
                        self.instruction_enter_immediate_value_to_register(0, MixalRegister::RA);
                        self.instruction_load_address_sign_to_register(identifier1_address, MixalRegister::RA);
                        operator_fn(self, identifier2_address);
                    } else if let (Token::Num(number), Token::Id(identifier)) = (left_operand.value(), right_operand.value()){
                        self.instruction_enter_immediate_value_to_register(0, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(*number, MixalRegister::RX);
                        let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
                        operator_fn(self, identifier_address);
                    } else if let (Token::Id(identifier), Token::Num(number)) = (left_operand.value(), right_operand.value()) {
                        let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
                        self.instruction_enter_immediate_value_to_register(*number, MixalRegister::RA);
                        self.instruction_store_register_to_address(0, MixalRegister::RA);
                        self.instruction_enter_immediate_value_to_register(0, MixalRegister::RA);
                        self.instruction_load_address_sign_to_register(identifier_address, MixalRegister::RA);
                        self.instruction_load_address_to_register(identifier_address, MixalRegister::RX);
                        operator_fn(self, 0);
                    }
                    if let Token::Percent = node.value() {
                        self.instruction_store_register_to_address(0, MixalRegister::RX);
                        self.instruction_load_address_to_register(0, MixalRegister::RA);
                    }                    
                }                
                // TODO: handle the rest of the cases here
                _ => {}
            }
            return;
        }                
        // TODO: handle the rest of the cases here
    }

    fn write_to_file(&mut self, str: String) {
        self.file.write_all(str.as_bytes()).expect("to be written");
    }

    fn token_to_arithmetic_operator_instruction_fn(token: &Token) -> fn(&mut MixalAssembler, u16) {
        match token {
            Token::Plus => MixalAssembler::instruction_add,
            Token::Minus => MixalAssembler::instruction_subtract,
            Token::Asterisk => MixalAssembler::instruction_multiply,
            Token::Slash | Token::Percent => MixalAssembler::instruction_divide_and_modulo,
            _ => MixalAssembler::instruction_add
        }        
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

    fn instruction_load_address_to_register(&mut self, address: u16, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_load_mnemonic(register),
            Some(String::from(format!("{}(0:5)", address)))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_load_address_sign_to_register(&mut self, address: u16, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_load_mnemonic(register),
            Some(String::from(format!("{}(0:0)", address)))
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

    fn instruction_store_register_sign_to_address(&mut self, address: u16, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_store_mnemonic(register),
            Some(String::from(format!("{}(0:0)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_store_register_without_sign_to_address(&mut self, address: u16, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_store_mnemonic(register),
            Some(String::from(format!("{}(1:5)", address.to_string())))
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

    fn instruction_enter_immediate_value_to_register(&mut self, value: i32, register: MixalRegister) {
        let mut instruction = MixalInstruction::new(
            None,
            mixal_register_to_enter_mnemonic(register, value),
            Some(String::from(value.abs().to_string()))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_add(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::ADD, 
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_subtract(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::SUB, 
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_multiply(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::MUL,
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_divide_and_modulo(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::DIV,
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }
}
