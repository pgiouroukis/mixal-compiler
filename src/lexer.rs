use logos::{Lexer, Logos};

// Definition of the language's tokens.
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n]+")] // Ignore this regex pattern between tokens
pub enum Token {
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,

    #[token("var")]
    Var,
    #[token("int")]
    Int,

    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("!")]
    ExclamationMark,

    #[token("=")]
    Assignment,

    #[token("+=")]
    AdditionAssignment,
    #[token("-=")]
    SubtractionAssignment,
    #[token("*=")]
    MultiplicationAssignment,
    #[token("/=")]
    DivisionAssignment,
    #[token("%=")]
    ModuloAssignment,

    #[token("==")]
    Equals,
    #[token("!=")]
    NotEquals,
    #[token("<")]
    LessThan,
    #[token(">")]
    GreaterThan,
    #[token("<=")]
    LessThanOrEquals,
    #[token(">=")]
    GreaterThanOrEquals,

    #[token("&&")]
    And,
    #[token("||")]
    Or,

    #[regex("[a-zA-Z]([a-zA-Z]|[0-9]|_)*", to_string)]
    Id(String),
    #[regex("([1-9][0-9]*)|0", to_num)]
    Num(u32),

    // The following variant is not a token.
    // We use it as a value in AST nodes
    // in order to group together tokens,
    // for example blocks of statements
    Ast(String)    
}

fn to_string(lex: &mut Lexer<Token>) -> Option<String> {
    let string: String = lex.slice().to_string();
    Some(string)
}

fn to_num(lex: &mut Lexer<Token>) -> Option<u32> {
    Some(lex.slice().parse().ok()?)
}
