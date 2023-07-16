use std::mem;
use crate::lexer::Token;

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
        return self.program_rule().tokens_consumed == self.tokens.len();
    }

    fn program_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::LeftBrace),
                Rhs::Nonterminal(Parser::decls_rule),
                Rhs::Terminal(Token::RightBrace)
            ]
        ], false);
    }

    fn decls_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::decl_rule),
                Rhs::Nonterminal(Parser::decls_rule)
            ]
        ], true);
    }

    fn decl_rule(&mut self) -> RuleResult {
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

    fn vars_rule(&mut self) -> RuleResult {
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
            return true;
        }
        return false;
    }

    fn next_token(&mut self) {
        self.pos += 1;
    }

    fn back_n_tokens(&mut self, n: usize) {
        self.pos -= n;
    }

    fn run_rules_from_rhs(&mut self, rhs: Vec<Vec<Rhs>>, contains_epsilon: bool) -> RuleResult {
        for rule in rhs.iter() {
            let rule_result = self.run_single_rule_from_rhs(rule);
            if rule_result.matched {
                return RuleResult{
                    matched: true, 
                    tokens_consumed: rule_result.tokens_consumed
                };
            }
        }
        return RuleResult{matched: contains_epsilon, tokens_consumed: 0};
    }

    // Consider the scenario in which a rhs is matched all the way
    // until the last element. This means that we have consumed 
    // all but the last tokens, but the rule itself failed and returned false.
    // The caller of this function (i.e. a Nonterminal production rule) 
    // will receive false. If the caller does not contain an epsilon rule,
    // then the code works as expected, since the caller will also fail and
    // it will propagate the message. But, what if the caller contains an epsilon
    // rule. Then, we consumed all but the last tokens of this rule, but the
    // rule was not eventually matched. In this method, we handle that by keeping
    // track of the consumed tokens and 'returning them back' if the rule
    // is not matched.
    fn run_single_rule_from_rhs(&mut self, rhs:&Vec<Rhs>) -> RuleResult {
        let mut count_tokens_matched = 0;
        for rhs_element in rhs.iter() {
            match rhs_element {
                Rhs::Terminal(token) => {
                    if !self.current_token_matches(&token) {
                        return RuleResult{
                            matched: false, 
                            tokens_consumed: count_tokens_matched
                        }
                    }
                    count_tokens_matched += 1;
                    self.next_token();
                },
                Rhs::Nonterminal(fn_to_run) => {
                    let rule_result = fn_to_run(self);
                    if !rule_result.matched  {
                        self.back_n_tokens(rule_result.tokens_consumed);
                        return RuleResult {
                            matched: false,
                            tokens_consumed: count_tokens_matched
                        }
                    }
                    count_tokens_matched += rule_result.tokens_consumed;
                }
            }
        }

        return RuleResult{
            matched: true,
            tokens_consumed: count_tokens_matched
         };
    }

}

// This struct models the result of an attempt to match a rule.
pub struct RuleResult {
    pub matched: bool,          // Whether the rule was matched or not
    pub tokens_consumed: usize  // How many tokens did the rule consume until matched or failure
}

pub enum Rhs {
    Terminal(Token),
    Nonterminal(fn(&mut Parser) -> RuleResult)
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

    #[test]
    fn test_failed_parsing_no_semicolon_at_eol() {
        let program = String::from(
            "{ \
                var first, second, third : int \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), false);
    }    
}
