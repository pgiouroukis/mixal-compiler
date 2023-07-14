use std::mem;
use crate::lexer::Token;

const MAX_RECURSION_DEPTH: usize = 5;

// Implementation of the language's parser.
// You can check the grammar of the language
// in <repo_root>/docs/grammar.txt
#[derive(Debug)]
pub struct Parser {
    pub pos: usize,
    pub tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { pos: 0, tokens }
    }

    pub fn analyze_grammar(&mut self) -> bool {
        // TODO: check for any remaining unconsumed tokens after the parsing is finished
        return self.program_rule();
    }

    fn program_rule(&mut self) -> bool {
        if 
            self.current_token_matches(&Token::LeftBrace) &&
            self.decls_rule(0) &&
            self.current_token_matches(&Token::RightBrace)
        {
            return true;
        }
        return false;
    }

    // This rule is left-recursive, so it may loop forever. We (temporarily)
    // handle this by using the MAX_RECURSION_DEPTH constant to allow only 
    // a specific amount of recursion depth. This is not correct, 
    // since it sets a limit on the amount of productions for this rule.
    // To fix this, we need to make some changes to the grammar in order
    // to eliminate any left-recursiveness. For now, we let it as is
    // and we will fix it in the future. Once fixed, we can use the generic
    // method 'Parser::run_rules_from_rhs' to express the rule and be
    // consistent with the rest of the rules
    // TODO: eliminate left-recursive rules from the grammar and fix
    //       rules that are recursive
    fn decls_rule(&mut self, depth: usize) -> bool {
        if depth > MAX_RECURSION_DEPTH {
            return false;
        }
        if 
            self.decls_rule(depth + 1) &&
            self.decl_rule() 
        {
            return true; 
        }
        return true;
    }

    fn decl_rule(&mut self) -> bool {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::Var),
                Rhs::Terminal(Token::Id(String::from("_"))),
                Rhs::Nonterminal(Parser::vars_rule),
                Rhs::Terminal(Token::Colon),
                Rhs::Terminal(Token::Int),
                Rhs::Terminal(Token::Semicolon)
            ]
        ], false);
    }

    fn vars_rule(&mut self) -> bool {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::Comma),
                Rhs::Terminal(Token::Id(String::from("_"))),
                Rhs::Nonterminal(Parser::vars_rule)
            ]
        ], true);
    }

    fn current_token_matches(&mut self, token: &Token) -> bool {
        // Using `mem::discriminant` instead of `==` because rust will compare 
        // both the enum variant AND the data contained in the variant (if applicable)
        // We don't want this behaviour here, since we only care about the enum variant equality
        if mem::discriminant(&self.tokens[self.pos]) == mem::discriminant(token) {
            println!("consumed: {:?}", self.tokens[self.pos]);
            self.next_token();
            return true;
        }
        return false;
    }

    fn next_token(&mut self) {
        self.pos += 1;
    }

    fn run_rules_from_rhs(&mut self, rhs: Vec<Vec<Rhs>>, contains_epsilon: bool) -> bool {
        for rule in rhs.iter() {
            if self.run_single_rule_from_rhs(rule) {
                return true;
            }
        }
        return contains_epsilon;
    }

    fn run_single_rule_from_rhs(&mut self, rhs:&Vec<Rhs>) -> bool {
        let mut did_match_token = true;
        for rhs_element in rhs.iter() {
            match rhs_element {
                Rhs::Terminal(token) => {
                    if !self.current_token_matches(&token) {
                        did_match_token = false;
                        break;
                    }
                },
                Rhs::Nonterminal(fn_to_run) => {
                    if !fn_to_run(self)  {
                        did_match_token = false;                        
                        break;
                    }
                }
            }
        }
        // We don't check 'contains_epsilon_rule' at this method
        // because we want to consume as many tokens as possible
        return did_match_token;
    }

}

pub enum Rhs {
    Terminal(Token),
    Nonterminal(fn(&mut Parser) -> bool)
}

// ------------------------------------------------------
//                        TESTS 
// ------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utilities::get_tokens_from_program;

    #[test]
    fn test_empty_square_brackets() {
        let program = String::from(
            "{}",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }

    #[test]
    fn test_one_decl_line_one_variable() {
        let program = String::from(
            "{ \
                var first : int; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }

    #[test]
    fn test_one_decl_line_multiple_variables() {
        let program = String::from(
            "{ \
                var first, second, third : int; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }

    #[test]
    fn test_multiple_decl_lines_one_variable() {
        let program = String::from(
            "{ \
                var first : int; \
                var second: int; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }

    #[test]
    fn test_multiple_decl_lines_multiple_variables() {
        let program = String::from(
            "{ \
                var first, second : int; \
                var third: int; \
                var fourt, fifth: int; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }    

    #[test]
    fn test_failed_parsing_no_colon() {
        let program = String::from(
            "{ \
                var first, second, third int; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), false);
    }
}