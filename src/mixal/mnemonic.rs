#[derive(Debug)]
pub enum MixalMnemonic {
    // Assembly pseudoinstructions
    ORIG,
    END,
    NOP,

    // MIX machine instructions
    LDA,
    LDX,
    LD1,
    STA,
    STX,
    ST1,
    ENTA,
    ENTX,
    ENT1,
    ENNA,
    ENNX,
    ENN1,    
    STZ,
    ADD,
    SUB,
    MUL,
    DIV,
    CMPA,
    CMPX,
    JSJ,
    JL,
    JE,
    JG,
    JGE,
    JNE,
    JLE,
    JAN,
    CHAR,
    OUT,
}
