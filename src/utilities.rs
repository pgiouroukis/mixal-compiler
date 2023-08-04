use crate::lexer::Token;
use logos::Logos; // defines Token::lexer so it must be imported, read more here: https://stackoverflow.com/questions/25273816/why-do-i-need-to-import-a-trait-to-use-the-methods-it-defines-for-a-type
use orange_trees::Node;

pub fn get_tokens_from_program(program: &String) -> Vec<Token> {
    let mut lex = Token::lexer(&program);
    let mut tokens = Vec::new();
    loop {
        let iter = lex.next();
        match iter {
            None => break,
            Some(val) => tokens.push(val.unwrap())
        }
    }
    return tokens;
}

pub fn new_node_from_token(token_index: usize, token: Token) ->  Node<usize, Token> {
    return Node::<usize, Token>::new(
        token_index,
        token
    );
}