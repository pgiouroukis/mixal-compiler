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
