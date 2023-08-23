#[derive(Debug)]
pub enum MixalMnemonic {
    // Assembly pseudoinstructions
    ORIG,
    END,
    NOP,

    // MIX machine instructions
    LDA,
    LDX,
    STA,
    STX,
    ENTA,
    ENTX,
    ENNA,
    ENNX,    
    STZ,
    ADD,
    SUB,
    MUL,
    DIV,
    CMPA,
    JL,
    JE,
    JG,
    JGE,
    JNE,
    JLE,
}
