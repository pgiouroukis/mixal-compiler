use std::process::Command;

use crate::lexer::Token;
use super::{register::MixalRegister, mnemonic::MixalMnemonic};

pub fn mixal_register_to_load_mnemonic(register: MixalRegister) -> MixalMnemonic {
    match register {
        MixalRegister::RA => return MixalMnemonic::LDA,
        MixalRegister::RX => return MixalMnemonic::LDX,
        MixalRegister::RI1 => return MixalMnemonic::LD1
    }    
}

pub fn mixal_register_to_store_mnemonic(register: MixalRegister) -> MixalMnemonic {
    match register {
        MixalRegister::RA => return MixalMnemonic::STA,
        MixalRegister::RX => return MixalMnemonic::STX,
        MixalRegister::RI1 => return MixalMnemonic::ST1
    }
}

pub fn mixal_register_to_enter_mnemonic(register: MixalRegister, value: i32) -> MixalMnemonic {
    match register {
        MixalRegister::RA => if value >= 0 { MixalMnemonic::ENTA } else { MixalMnemonic::ENNA },
        MixalRegister::RX => if value >= 0 { MixalMnemonic::ENTX } else { MixalMnemonic::ENNX },
        MixalRegister::RI1 => if value >= 0 { MixalMnemonic::ENT1 } else { MixalMnemonic::ENN1 },
    }    
}

pub fn comparison_token_to_jump_instruction(token: Token) -> MixalMnemonic {
    match token {
        Token::Equals => MixalMnemonic::JE,
        Token::NotEquals => MixalMnemonic::JNE,
        Token::LessThan => MixalMnemonic::JL,
        Token::GreaterThan => MixalMnemonic::JG,
        Token::LessThanOrEquals => MixalMnemonic::JLE,
        Token::GreaterThanOrEquals => MixalMnemonic::JGE,
        _ => MixalMnemonic::JE
    }
}

pub fn get_random_instruction_label() -> String {
    let charset: String = ('A'..'Z').map(|c| c as char).collect();
    return random_string::generate(3, charset.clone());
}

pub fn run_mix_binary_file_and_print_output(file_path: &String) {
    Command::new("mixvm")
        .arg("--run")
        .arg(file_path)
        .output()
        .expect("to execute")
        .stdout
        .iter()
        .for_each(|x| print!("{}", *x as char));    
}
