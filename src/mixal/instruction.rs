use super::mnemonic::MixalMnemonic;

pub struct MixalInstruction {
    pub label: Option<String>,
    pub mnemonic: MixalMnemonic,
    pub operand: Option<String>,
}

impl MixalInstruction {
    pub fn new(
        label: Option<String>, 
        mnemonic: MixalMnemonic, 
        operand: Option<String>
    ) -> MixalInstruction{
        MixalInstruction {
            label, mnemonic, operand
        }
    }

    pub fn to_string(&mut self) -> String {
        let mut label_value = String::from("");
        if let Option::Some(value) = &self.label {
            label_value = value.clone();
        };

        let mnemonic_value = format!("{:?}", self.mnemonic);

        let mut operand_value = String::from("");
        if let Option::Some(value) = &self.operand {
            operand_value = value.to_string();
        };

        return format!("{} {} {}\n", label_value, mnemonic_value, operand_value);
    }
}
