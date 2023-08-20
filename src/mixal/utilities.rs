use super::{register::MixalRegister, mnemonic::MixalMnemonic};

pub fn mixal_register_to_load_mnemonic(register: MixalRegister) -> MixalMnemonic {
    match register {
        MixalRegister::RA => return MixalMnemonic::LDA,
        MixalRegister::RX => return MixalMnemonic::LDX
    }    
}

pub fn mixal_register_to_store_mnemonic(register: MixalRegister) -> MixalMnemonic {
    match register {
        MixalRegister::RA => return MixalMnemonic::STA,
        MixalRegister::RX => return MixalMnemonic::STX,
    }
}

pub fn mixal_register_to_enter_mnemonic(register: MixalRegister, value: i32) -> MixalMnemonic {
    match register {
        MixalRegister::RA => if value >= 0 { MixalMnemonic::ENTA } else { MixalMnemonic::ENNA },
        MixalRegister::RX => if value >= 0 { MixalMnemonic::ENTX } else { MixalMnemonic::ENNX },
    }    
}
