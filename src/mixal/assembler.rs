use orange_trees::Node;
use std::{fs::File, io::Write};
use std::process::Command;
use std::collections::HashMap;
use crate::lexer::Token;
use crate::utilities::arithmetic_assignment_operator_to_arithmetic_operator;
use super::{instruction::*, mnemonic::*, register::*, utilities::*};

const PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS: u16 = 2000;

pub struct MixalAssembler {
    pub ast: Node<usize, Token>,
    pub output_file_path: String,
    file: File,
    vtable: HashMap<String, u16>,
    next_memory_address_to_allocate: u16,
    loop_stack: Vec<(String,String)>
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
            next_memory_address_to_allocate: 1,
            loop_stack: vec![]
        }
    }

    pub fn run(&mut self) {
        self.instruction_set_instructions_allocation_address(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        self.handle_root(self.ast.clone());        
        self.instruction_end_program(PROGRAM_INSTRUCTIONS_ALLOCATION_ADDRESS);
        Command::new("mixasm").arg(self.output_file_path.clone()).output().expect("to execute");
    }

    fn handle_root(&mut self, node: Node<usize, Token>) {        
        match node.value() {
            Token::Ast(_) => {
                let children = node.children();
                for child in children {
                    self.handle_root(child.clone());
                }
            },
            Token::Int => {
                self.handle_variable_declaration(node.clone());
            },
            Token::Assignment => {
                self.handle_assignment_operator(node.clone());
            },
            Token::AdditionAssignment | Token::SubtractionAssignment
            | Token::MultiplicationAssignment | Token::DivisionAssignment
            | Token::ModuloAssignment => {
                self.handle_arithmetic_assignment_operator(node.clone())
            },
            Token::If => {
                self.handle_if_statement(node.clone());
            },
            Token::While => {
                self.handle_while_loop(node.clone())
            },
            Token::For => {
                self.handle_for_loop(node.clone());
            },
            Token::Continue => {
                let continue_label = self.loop_stack.last().expect("to exist").0.clone();
                self.instruction_jump_to_label(continue_label);
            },
            Token::Break => {
                let break_label = self.loop_stack.last().expect("to exist").1.clone();
                self.instruction_jump_to_label(break_label);
            },
            _ => {}
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

    fn handle_arithmetic_assignment_operator(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        let identifier_node = children.get(0).expect("to exist");
        let expression_node = children.get(1).expect("to exist");
        
        let mut new_expression_node = Node::new(
            0,
            arithmetic_assignment_operator_to_arithmetic_operator(node.value().clone())
        );
        new_expression_node.add_child(identifier_node.clone());
        new_expression_node.add_child(expression_node.clone());
        
        let mut new_assignment_node = Node::new(1, Token::Assignment);
        new_assignment_node.add_child(identifier_node.clone());
        new_assignment_node.add_child(new_expression_node.clone());

        self.handle_assignment_operator(new_assignment_node);
    }

    fn handle_if_statement(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        let expression_node = children.get(0).expect("to exist");
        self.handle_expression_node(expression_node.clone());

        let else_label = get_random_instruction_label();
        let bottom_label = get_random_instruction_label();
        
        self.instruction_store_zero_to_address(0);
        self.instruction_compare_ra(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::Equals, else_label.clone());
        let block_node = children.get(1).expect("to exist");
        self.handle_root(block_node.clone());
        self.instruction_jump_to_label(bottom_label.clone());

        self.instruction_nop_with_label(else_label.clone());
        if children.len() == 3 {
            let else_node = children.get(2).expect("to exist");
            let else_block_node = else_node.children().get(0).expect("to exist");
            self.handle_root(else_block_node.clone());
        }

        self.instruction_nop_with_label(bottom_label.clone());
    }

    fn handle_while_loop(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        let expression_node = children.get(0).expect("to exist");
        let code_block_node = children.get(1).expect("to exist");

        let evaluate_expression_label = get_random_instruction_label();
        let exit_loop_label = get_random_instruction_label();

        self.loop_stack.push((
            evaluate_expression_label.clone(),
            exit_loop_label.clone()
        ));

        self.instruction_nop_with_label(evaluate_expression_label.clone());
        self.handle_expression_node(expression_node.clone());
        self.instruction_store_zero_to_address(0);
        self.instruction_compare_ra(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::Equals, exit_loop_label.clone());
        self.handle_root(code_block_node.clone());
        self.instruction_jump_to_label(evaluate_expression_label.clone());
        self.instruction_nop_with_label(exit_loop_label.clone());

        self.loop_stack.pop();
    }

    fn handle_for_loop(&mut self, node: Node<usize, Token>) {
        let children = node.children();
        let assignment_node = children.get(0).expect("to exist");
        let expression_node = children.get(1).expect("to exist");
        let statement_node = children.get(2).expect("to exist");
        let code_block_node = children.get(3).expect("to exist");

        let evaluate_expression_label = get_random_instruction_label();
        let exit_loop_label = get_random_instruction_label();
        
        // In the case of a for loop, when we encounter 'continue', we still
        // need to run the 3rd part of the loop ('statement_node'). Because of
        // this, we will also need a label so we can skip the rest of the loop's 
        // code but still execute the 'statement_node'. We define that label here.
        let evaluate_expression_label_for_continue = get_random_instruction_label();

        self.loop_stack.push((
            evaluate_expression_label_for_continue.clone(),
            exit_loop_label.clone()
        ));

        self.handle_root(assignment_node.clone());
        self.instruction_nop_with_label(evaluate_expression_label.clone());
        self.handle_expression_node(expression_node.clone());
        self.instruction_store_zero_to_address(0);
        self.instruction_compare_ra(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::Equals, exit_loop_label.clone());
        self.handle_root(code_block_node.clone());
        self.instruction_nop_with_label(evaluate_expression_label_for_continue.clone());
        self.handle_root(statement_node.clone());
        self.instruction_jump_to_label(evaluate_expression_label.clone());
        self.instruction_nop_with_label(exit_loop_label.clone());

        self.loop_stack.pop();
    }

    // Evaluates the expression starting from `node`
    // and stores the result in register RA
    fn handle_expression_node(&mut self, node: Node<usize, Token>) {
        if let Token::Num(number) = node.value() {
            self.instruction_enter_two_byte_immediate_value_to_register(*number, MixalRegister::RA);
            return;
        } else if let Token::Id(identifier) = node.value() {
            self.instruction_load_address_to_register(
                self.vtable.get(identifier).expect("to exist").clone(),
                MixalRegister::RA
            );
            return;
        } else if let Token::ExclamationMark = node.value() {
            let child = node.children().get(0).expect("to exist");
            self.handle_expression_node(child.clone());
            self.instructions_logical_not();
            return;
        }

        let children = node.children();
        let left_operand = children.get(0).expect("to exist");
        let right_operand = children.get(1).expect("to exist");

        if left_operand.is_leaf() && right_operand.is_leaf() {
            // In this branch, both of the operands are values, so we
            // can directly evaluate the operator result between them.
            // 
            // In MIX, all the arithmetic and comparison operators 
            // expect the left operand to be stored in register RA 
            // and the right operand to be stored in a memory address.
            // The only exception to this rule are the division and
            // modulo operators (instruction DIV), which expect the 
            // operand's MSBs to be stored in RA and LSBs to be stored 
            // in RX. Because of this exception, we handle division and 
            // modulo operators differently from the rest of the operators.
            //  
            // After evaluating a result, we store it in register
            // RA so it becomes available for future instructions.
        
            if let Token::Slash | Token::Percent = node.value() {
                self.instructions_prepare_leaf_operands_and_execute_division(
                    node.value(),
                    left_operand.value(),
                    right_operand.value(),
                );
            } else {
                self.instructions_prepare_leaf_operands_and_execute_operator(
                    node.value(),
                    left_operand.value(),
                    right_operand.value()
                );
            }
        } else {
            // In this branch, one or both of the operands are other
            // expressions. This means that we first have to evaluate
            // the expressions in the operands and then evaluate the 
            // operator result between them.
            // 
            // Initially, I thought that by using exclusively CPU registers
            // (without leveraging extra memory), any expression could
            // be calculated. But this is not possible for complicated
            // expressions, because at some point it is required to store
            // intermediate values (ie evaluated left node) while evaluating
            // the other node. But, the latter node might also need to store
            // intermediate values and so on. To solve this, we need to
            // utilize memory. This is how it is handled below. Upon
            // evaluating a child expression, we temporarily store it in memory.
            // After we evaluate the operator result, we de-alloate the memory.
            // 
            // Note that for simple expressions, we could still exclusively
            // use registers, but we don't implement it this way since the
            // code that decides that could be very comlicated.

            self.handle_expression_node(right_operand.clone());
            let temp_memory_address = self.next_memory_address_to_allocate;
            self.next_memory_address_to_allocate += 1;
            self.instruction_store_register_to_address(
                temp_memory_address,
                MixalRegister::RA
            );
            self.handle_expression_node(left_operand.clone());

            // As explained above, division and modulo require some special treatment
            if let Token::Slash | Token::Percent = node.value() {
                self.instruction_store_register_to_address(0, MixalRegister::RA);
                self.instruction_load_address_to_register(0, MixalRegister::RX);
                self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
                self.instruction_load_address_sign_to_register(0, MixalRegister::RA);                
            }
            
            let operator_fn = 
                MixalAssembler::token_to_arithmetic_operator_instruction_fn(node.value());
            operator_fn(self, temp_memory_address);

            self.next_memory_address_to_allocate -= 1;            
        }

        // At this point, the operator result is evaluated.
        // However, for some operators it is required to run
        // extra instructions in order to load the result in
        // register RA. This is what's handled here.
        match node.value() {
            Token::Plus | Token::Minus 
            | Token::And | Token::Or => {
                // No need to do anything for these operators,
                // the result is alredy loaded in RA
            },
            Token::Asterisk => {
                // RA contains the upper bits of the result and 
                // RX contains the lower bits of the result. 
                // The sign of the result is stored in the sign bit of RA.
                // For now, we don't handle overflows and only care about
                // the lower bits of the result, so we need to move RX to
                // RA, but we have to make sure we don't overwrite the sign.
                self.instructions_move_register_without_sign_to_register(
                    MixalRegister::RX,
                    MixalRegister::RA
                );
                // TODO: add code that throws exception when the result overflows                    
            },
            Token::Percent => {
                // RA contains the result of the division operator and
                // RX contains the result of the modulo operator.
                self.instructions_move_register_to_register(
                    MixalRegister::RX,
                    MixalRegister::RA                        
                );
            },
            Token::Equals | Token::NotEquals
            | Token::LessThan | Token::LessThanOrEquals
            | Token::GreaterThan | Token::GreaterThanOrEquals => {
                self.instructions_load_comparison_result_to_register_ra(node.value().clone());
            },
            _ => {}
        }
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
            Token::Equals | Token::NotEquals
            | Token::LessThan | Token::LessThanOrEquals
            | Token::GreaterThan | Token::GreaterThanOrEquals => MixalAssembler::instruction_compare_ra,
            Token::And => MixalAssembler::instructions_logical_and,
            Token::Or => MixalAssembler::instructions_logical_or,
            _ => MixalAssembler::instruction_add
        }        
    }

    // ---------------------------------------------
    //             MIXAL INSTRUCTIONS              
    //  The methods below model MIXAL instructions.
    //  They are prefixed with:
    //   - "instruction_" when they generate a
    //     single instruction.
    //   - "instructions_" when they generate
    //     a group of instructions.
    //  When invoked, they will append the needed
    //  instruction(s) to 'self.output_file_path'
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

    fn instruction_nop_with_label(&mut self, label: String) {
        let mut instruction = MixalInstruction::new(
            Some(label),
            MixalMnemonic::NOP,
            None
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

    fn instruction_enter_two_byte_immediate_value_to_register(&mut self, value: i32, register: MixalRegister) {
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

    fn instruction_compare_ra(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::CMPA,
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_compare_rx(&mut self, address: u16) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::CMPX,
            Some(String::from(format!("{}(0:5)", address.to_string())))
        );
        self.write_to_file(instruction.to_string());
    }

    fn instruction_jump_to_label(&mut self, label: String) {
        let mut instruction = MixalInstruction::new(
            None, 
            MixalMnemonic::JSJ,
            Some(label)
        );
        self.write_to_file(instruction.to_string());        
    }

    fn instruction_jump_to_label_if_comparison_was_true(&mut self, comparison_token: Token, label: String) {
        let mut instruction = MixalInstruction::new(
            None, 
            comparison_token_to_jump_instruction(comparison_token),
            Some(label)
        );
        self.write_to_file(instruction.to_string());        
    }

    fn instructions_prepare_leaf_operands_and_execute_operator(
        &mut self,
        operator: &Token,
        left_operand: &Token, 
        right_operand: &Token
    ) {
        let operator_fn = 
            MixalAssembler::token_to_arithmetic_operator_instruction_fn(operator);

        // Because an operand can be either a Number or a Variable, 
        // we must handle 4 cases, one for every combination.
        if let (Token::Num(number1), Token::Num(number2)) = (left_operand, right_operand) {
            self.instruction_enter_two_byte_immediate_value_to_register(*number2, MixalRegister::RA);
            self.instruction_store_register_to_address(0, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(*number1, MixalRegister::RA);
            operator_fn(self, 0);
        } else if let (Token::Id(identifier1), Token::Id(identifier2)) = (left_operand, right_operand) {
            let identifier1_address = self.vtable.get(identifier1).expect("to exist").clone();
            let identifier2_address = self.vtable.get(identifier2).expect("to exist").clone();
            self.instruction_load_address_to_register(identifier1_address, MixalRegister::RA);
            operator_fn(self, identifier2_address);
        } else if let (Token::Num(number), Token::Id(identifier)) = (left_operand, right_operand) {
            self.instruction_enter_two_byte_immediate_value_to_register(number.clone(), MixalRegister::RA);
            operator_fn(
                self,
                self.vtable.get(identifier).expect("to exist").clone()
            );
        } else if let (Token::Id(identifier), Token::Num(number)) = (left_operand, right_operand) {
            let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
            self.instruction_load_address_to_register(identifier_address, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(*number, MixalRegister::RX);
            self.instruction_store_register_to_address(0, MixalRegister::RX);
            operator_fn(self, 0);
        }
    }

    fn instructions_prepare_leaf_operands_and_execute_division(
        &mut self,
        operator: &Token,
        left_operand: &Token, 
        right_operand: &Token        
    ) {
        // In the AST, the Token::Num does not store negative numbers.
        // The only exception to this occurs when an expression uses
        // the unary minus operator `-x`. In this case, the AST transforms
        // `-x` to `(-1) * x`, and thus needs to store `Token::Num(-1)`.
        // But this case is handled in the multiplication operator case.
        // So, in the code below, we assume that every Token::Num is positive.
        // Note that the value of a `Token::Id` in memory CAN be negative.
        
        let operator_fn = 
            MixalAssembler::token_to_arithmetic_operator_instruction_fn(operator);
        
        // Because an operand can be either a Number or a Variable, 
        // we must handle 4 cases, one for every combination.
        if let (Token::Num(number1), Token::Num(number2)) = (left_operand, right_operand) {
            self.instruction_enter_two_byte_immediate_value_to_register(*number2, MixalRegister::RA);
            self.instruction_store_register_to_address(0, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(*number1, MixalRegister::RX);
            operator_fn(self, 0);
        } if let (Token::Id(identifier1), Token::Id(identifier2)) = (left_operand, right_operand) {
            let identifier1_address = self.vtable.get(identifier1).expect("to exist").clone();
            let identifier2_address = self.vtable.get(identifier2).expect("to exist").clone();
            self.instruction_load_address_to_register(identifier1_address, MixalRegister::RX);
            self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
            self.instruction_load_address_sign_to_register(identifier1_address, MixalRegister::RA);
            operator_fn(self, identifier2_address);
        } else if let (Token::Num(number), Token::Id(identifier)) = (left_operand, right_operand){
            self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(*number, MixalRegister::RX);
            let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
            operator_fn(self, identifier_address);
        } else if let (Token::Id(identifier), Token::Num(number)) = (left_operand, right_operand) {
            let identifier_address = self.vtable.get(identifier).expect("to exist").clone();
            self.instruction_enter_two_byte_immediate_value_to_register(*number, MixalRegister::RA);
            self.instruction_store_register_to_address(0, MixalRegister::RA);
            self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
            self.instruction_load_address_sign_to_register(identifier_address, MixalRegister::RA);
            self.instruction_load_address_to_register(identifier_address, MixalRegister::RX);
            operator_fn(self, 0);
        }
    }

    fn instructions_move_register_to_register(
        &mut self,
        origin_register: MixalRegister,
        destination_register: MixalRegister        
    ) {
        self.instruction_store_register_to_address(0, origin_register);
        self.instruction_load_address_to_register(0, destination_register);
    }

    fn instructions_move_register_without_sign_to_register(
        &mut self,
        origin_register: MixalRegister,
        destination_register: MixalRegister
    ) {
        self.instruction_store_register_sign_to_address(0, destination_register.clone());
        self.instruction_store_register_without_sign_to_address(0, origin_register);
        self.instruction_load_address_to_register(0, destination_register);
    }

    fn instructions_load_comparison_result_to_register_ra(&mut self, comparison_token: Token) {
        let label = get_random_instruction_label();

        self.instruction_enter_two_byte_immediate_value_to_register(1, MixalRegister::RA);
        self.instruction_jump_to_label_if_comparison_was_true(
            comparison_token, 
            label.clone()
        );
        self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
        self.instruction_nop_with_label(label.clone());
    }

    fn instructions_logical_and(&mut self, address: u16) {
        let bottom_label = get_random_instruction_label();

        // Assume that the result is true
        self.instruction_enter_two_byte_immediate_value_to_register(1, MixalRegister::RI1);

        // store operand2 to RX
        self.instruction_load_address_to_register(address, MixalRegister::RX);

        // store 0 to memory address 0
        self.instruction_store_zero_to_address(0);

        // if RA is zero, set result to 0 and don't check RX
        let label = get_random_instruction_label();
        self.instruction_compare_ra(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::NotEquals, label.clone());
        self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RI1);
        self.instruction_jump_to_label(bottom_label.clone());
        self.instruction_nop_with_label(label.clone());

        // if RX is zero, set result to 0
        let label = get_random_instruction_label();
        self.instruction_compare_rx(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::NotEquals, label.clone());
        self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RI1);
        self.instruction_nop_with_label(label.clone());

        self.instruction_nop_with_label(bottom_label.clone());
        self.instructions_move_register_to_register(MixalRegister::RI1, MixalRegister::RA);
    }

    fn instructions_logical_or(&mut self, address: u16) {
        let label_true = get_random_instruction_label();
        let label_bottom = get_random_instruction_label();

        // Assume that the result is false
        self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RI1);

        // Store operand2 to RX
        self.instruction_load_address_to_register(address, MixalRegister::RX);

        // Store 0 to memory address 0
        self.instruction_store_zero_to_address(0);

        // If RA != 0, jump to 'label_true'
        self.instruction_compare_ra(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::NotEquals, label_true.clone());
    
        // If RA != 0, jump to 'label_true'
        self.instruction_compare_rx(0);
        self.instruction_jump_to_label_if_comparison_was_true(Token::NotEquals, label_true.clone());

        // If this instruction is reached, it means that
        // the expression is false. Jump to 'label_bottom'
        self.instruction_jump_to_label(label_bottom.clone());

        self.instruction_nop_with_label(label_true.clone());
        self.instruction_enter_two_byte_immediate_value_to_register(1, MixalRegister::RI1);

        self.instruction_nop_with_label(label_bottom.clone());

        self.instructions_move_register_to_register(MixalRegister::RI1, MixalRegister::RA);
    }

    fn instructions_logical_not(&mut self) {
        let label = get_random_instruction_label();
    
        self.instruction_store_zero_to_address(0);
        self.instruction_compare_ra(0);
        self.instruction_enter_two_byte_immediate_value_to_register(1, MixalRegister::RA);
        self.instruction_jump_to_label_if_comparison_was_true(Token::Equals, label.clone());
        self.instruction_enter_two_byte_immediate_value_to_register(0, MixalRegister::RA);
        self.instruction_nop_with_label(label.clone());        
    }
}
