#[derive(Debug)]
pub enum MixalMnemonic {
    // Assembly pseudoinstructions
    ORIG,
    END,

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
}
