use std::mem;
use std::collections::HashMap;
use orange_trees::Node;
use crate::{lexer::Token, utilities::new_node_from_token};

// Implementation of the language's parser.
// You can check the grammar of the language
// in <repo_root>/docs/grammar.txt
#[derive(Debug)]
pub struct Parser {
    pub pos: usize,
    pub tokens: Vec<Token>,
    pub ast: Node<usize, Token>,

    // key: token_start_index
    // value: (token_end_index, Node)
    token_index_to_node: HashMap<usize, (usize, Node<usize, Token>)>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { 
            pos: 0, 
            tokens,
            ast: new_node_from_token(0, Token::Ast(String::from("ROOT_AST_NODE"))),
            token_index_to_node: HashMap::new()
        }
    }

    pub fn analyze_grammar(&mut self) -> bool {
        return self.program_rule().tokens_consumed == self.tokens.len();
    }

    fn program_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::LeftBrace),
                Rhs::Nonterminal(Parser::decls_rule),
                Rhs::Nonterminal(Parser::stmts_rule),
                Rhs::Terminal(Token::RightBrace)
            ]
        ], false);

        if rule_result.matched {
            let token_range_start = self.pos - rule_result.tokens_consumed;
            let token_range_end = self.pos;
            let mut index = token_range_start + 1;
            let mut node= new_node_from_token(token_range_start, Token::Ast(String::from("PROGRAM")));
            while  index < token_range_end - 1 {
                let child = self.token_index_to_node.get(&index).expect("has value").clone();                
                node.add_child(child.1.clone());
                index = child.0;
            }
            self.ast.add_child(node.clone());
        }
        
        return rule_result;
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
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::Var),
                Rhs::Terminal(Token::Id(String::from("_"))),
                Rhs::Nonterminal(Parser::vars_rule),
                Rhs::Terminal(Token::Colon),
                Rhs::Terminal(Token::Int),
                Rhs::Terminal(Token::Semicolon)
            ]
        ], false);

        if rule_result.matched {
            let mut node: Node<usize, Token>;
            let token_range_start = self.pos - rule_result.tokens_consumed;
            let token_range_end = self.pos;
            node = new_node_from_token(token_range_start, Token::Int);
            for i in token_range_start + 1 .. token_range_end {
                let token = (*self.tokens.get(i).clone().expect("has value")).clone();
                if let Token::Id(_) = token {
                    node.add_child(new_node_from_token(i, token.clone()));
                }
            }
            self.token_index_to_node.insert(
                token_range_start,
                (self.pos, node.clone())
            );
        }

        return rule_result;
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

    fn stmts_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::stmt_rule),
                Rhs::Nonterminal(Parser::stmts_rule)
            ]
        ], true)
    }

    fn stmt_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::simp_rule),
                Rhs::Terminal(Token::Semicolon)
            ],
            vec![Rhs::Nonterminal(Parser::control_rule)],
            vec![Rhs::Terminal(Token::Semicolon)]
        ], false);

        if rule_result.matched  && rule_result.tokens_consumed > 1 {
            let index = self.pos - rule_result.tokens_consumed;
            let node = self.token_index_to_node.get(&index).expect("has value").clone();
            self.token_index_to_node.insert(
                index,
                (self.pos, node.1.clone())
            );
        }

        return rule_result;
    }

    fn simp_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::Id(String::from("_"))),
                Rhs::Nonterminal(Parser::asop_rule),
                Rhs::Nonterminal(Parser::exp_rule),
            ],
            vec![
                Rhs::Terminal(Token::Print),
                Rhs::Nonterminal(Parser::exp_rule)
            ]
        ], false);

        if rule_result.matched {
            let index = self.pos - rule_result.tokens_consumed;
            let first_token = (*self.tokens.get(index).clone().expect("has value")).clone();
            let mut node = new_node_from_token(0, Token::Break);
            match first_token {
                Token::Id(_) => {
                    let assignment_operator = (*self.tokens.get(index+1).clone().expect("has value")).clone();                    
                    node = new_node_from_token(index+1, assignment_operator.clone());
                    node.add_child(new_node_from_token(index, first_token));
                    let expression_node = self.token_index_to_node.get(&(index+2)).expect("has value").clone();
                    node.add_child(expression_node.1);
                },
                Token::Print => {
                    node = new_node_from_token(index, Token::Print);
                    let expression_node = self.token_index_to_node.get(&(index+1)).expect("has value").clone();
                    node.add_child(expression_node.1);
                },
                _ => {}
            }
            self.token_index_to_node.insert(
                index,
                (self.pos, node.clone())
            );            
        }

        return rule_result;
    }

    fn control_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::If),
                Rhs::Terminal(Token::LeftParen),
                Rhs::Nonterminal(Parser::exp_rule),
                Rhs::Terminal(Token::RightParen),
                Rhs::Nonterminal(Parser::block_rule),
                Rhs::Nonterminal(Parser::else_block_rule),
            ],
            vec![
                Rhs::Terminal(Token::While),
                Rhs::Terminal(Token::LeftParen),
                Rhs::Nonterminal(Parser::exp_rule),
                Rhs::Terminal(Token::RightParen),
                Rhs::Nonterminal(Parser::block_rule),
            ],
            vec![
                Rhs::Terminal(Token::For),
                Rhs::Terminal(Token::LeftParen),
                Rhs::Nonterminal(Parser::simp_rule),
                Rhs::Terminal(Token::Semicolon),
                Rhs::Nonterminal(Parser::exp_rule),
                Rhs::Terminal(Token::Semicolon),
                Rhs::Nonterminal(Parser::simp_rule),
                Rhs::Terminal(Token::RightParen),
                Rhs::Nonterminal(Parser::block_rule),
            ],
            vec![Rhs::Terminal(Token::Continue), Rhs::Terminal(Token::Semicolon)],
            vec![Rhs::Terminal(Token::Break), Rhs::Terminal(Token::Semicolon)],
        ], false);


        if rule_result.matched {
            let index = self.pos - rule_result.tokens_consumed;
            let token = (*self.tokens.get(index).clone().expect("has value")).clone();
            let mut node = new_node_from_token(0, Token::Break);
            match token {
                Token::If => {
                    node = new_node_from_token(index, Token::If);
                    let expression_node = self.token_index_to_node.get(&(index+2)).expect("has value").clone();
                    node.add_child(expression_node.1);
                    let block_node = self.token_index_to_node.get(&(expression_node.0 + 1)).expect("has value").clone();
                    node.add_child(block_node.1.clone());
                    if block_node.0 < self.pos {
                        let else_block = self.token_index_to_node.get(&block_node.0).expect("has value").clone();
                        node.add_child(else_block.1.clone());
                    }
                },
                Token::While => {
                    node = new_node_from_token(index, Token::While);
                    let expression_node = self.token_index_to_node.get(&(index+2)).expect("has value").clone();
                    node.add_child(expression_node.1);
                    let block_node = self.token_index_to_node.get(&(expression_node.0 + 1)).expect("has value").clone();
                    node.add_child(block_node.1.clone());
                }
                Token::For => {
                    node = new_node_from_token(index, Token::For);
                    let simp_node = self.token_index_to_node.get(&(index+2)).expect("has value").clone();
                    node.add_child(simp_node.1);
                    let expression_node = self.token_index_to_node.get(&(simp_node.0 + 1)).expect("has value").clone();
                    node.add_child(expression_node.1);
                    let simp_node = self.token_index_to_node.get(&(expression_node.0 + 1)).expect("has value").clone();
                    node.add_child(simp_node.1);
                    let block_node = self.token_index_to_node.get(&(simp_node.0 + 1)).expect("has value").clone();
                    node.add_child(block_node.1.clone());
                }
                Token::Continue => {
                    node = new_node_from_token(index, Token::Continue);
                },
                Token::Break => {
                    node = new_node_from_token(index, Token::Break);
                },
                _ => {}
            }
            self.token_index_to_node.insert(
                index,
                (self.pos, node.clone())
            );
        }

        return rule_result;
    }

    fn block_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![Rhs::Nonterminal(Parser::stmt_rule)],
            vec![
                Rhs::Terminal(Token::LeftBrace),
                Rhs::Nonterminal(Parser::stmts_rule),
                Rhs::Terminal(Token::RightBrace)
            ],
        ], false);

        if rule_result.matched {
            let index_start = self.pos - rule_result.tokens_consumed;
            let first_token = (*self.tokens.get(index_start).clone().expect("has value")).clone();
            match first_token {
                Token::LeftBrace => {
                    let token_range_start = self.pos - rule_result.tokens_consumed;
                    let token_range_end = self.pos;
                    let mut index = token_range_start + 1;
                    let mut node= new_node_from_token(token_range_start, Token::Ast(String::from("BLOCK")));
                    while  index < token_range_end - 1 {
                        let child = self.token_index_to_node.get(&index).expect("has value").clone();
                        node.add_child(child.1.clone());
                        index = child.0;
                    }
                    self.token_index_to_node.insert(
                        index_start,
                        (self.pos, node.clone())
                    );
                },
                _ => {
                    let mut node= new_node_from_token(index_start, Token::Ast(String::from("SINGLE_BLOCK")));
                    let stmt_node = self.token_index_to_node.get(&index_start).expect("has value").clone();
                    node.add_child(stmt_node.1.clone());
                    self.token_index_to_node.insert(
                        index_start,
                        (self.pos, node.clone())
                    );
                }
            }
        }

        return rule_result;
    }

    fn else_block_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Terminal(Token::Else),
                Rhs::Nonterminal(Parser::block_rule)
            ]
        ], true);

        if rule_result.matched && rule_result.tokens_consumed > 0 {
            let index_start = self.pos - rule_result.tokens_consumed;
            let mut node= new_node_from_token(index_start, Token::Else);
            let block_node = self.token_index_to_node.get(&(index_start+1)).expect("has value").clone();
            node.add_child(block_node.1.clone());
            self.token_index_to_node.insert(
                index_start,
                (self.pos, node.clone())
            );            
        }

        return rule_result;
    }

    fn exp_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::precedence_2_rule),
                Rhs::Nonterminal(Parser::precedence_1_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 0 {
            println!("precedence_1_rule (exp) starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result
    }

    fn precedence_2_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::precedence_3_rule),
                Rhs::Nonterminal(Parser::precedence_2_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 1 {
            println!("precedence_2_rule starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result;
    }

    fn precedence_3_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::precedence_4_rule),
                Rhs::Nonterminal(Parser::precedence_3_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 1 {
            println!("precedence_3_rule starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result;
    }

    fn precedence_4_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::precedence_5_rule),
                Rhs::Nonterminal(Parser::precedence_4_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 1 {
            println!("precedence_4_rule starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result;   
    }

    fn precedence_5_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::precedence_6_rule),
                Rhs::Nonterminal(Parser::precedence_5_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 1 {
            println!("precedence_5_rule starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result;
    }

    fn precedence_6_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::unary_rule),
                Rhs::Nonterminal(Parser::precedence_6_recursive_rule),
            ],
        ], false);

        if rule_result.matched && rule_result.tokens_consumed > 1 {
            println!("precedence_6_rule starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            self.construct_expression_node_from_token_range(
                self.pos-rule_result.tokens_consumed,
                self.pos
            );
        }

        return rule_result;
    }

    fn unary_rule(&mut self) -> RuleResult {
        let rule_result = self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::unop_rule),
                Rhs::Nonterminal(Parser::base_rule),
            ],
            vec![Rhs::Nonterminal(Parser::base_rule)]
        ], false);

        if rule_result.matched && rule_result.tokens_consumed == 2 {
            // For cases like -3, -alpha, !a, !3
            println!("unary starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);
            let index = self.pos-rule_result.tokens_consumed;
            let unary_token = (*self.tokens.get(index).clone().expect("has value")).clone();
            let value_token = (*self.tokens.get(index+1).clone().expect("has value")).clone();
            let mut node = new_node_from_token(0, Token::Break);
            match unary_token {
                Token::Minus => {
                    node = new_node_from_token(index, Token::Asterisk);      
                    node.add_child(new_node_from_token(index+10001, Token::Num(0)));
                    node.add_child(new_node_from_token(index+10002, value_token.clone()));
                }, 
                Token::ExclamationMark => {
                    node = new_node_from_token(index, Token::ExclamationMark);
                    node.add_child(new_node_from_token(index+10001, value_token.clone()));
                }, 
                _ => {}
            }
            self.token_index_to_node.insert(
                index,
                (self.pos, node.clone())
            );                    
        } else if rule_result.matched && rule_result.tokens_consumed > 2 {
            // For cases like -(1+3), !(alpha+2).
            // Notice that the expression inside the parentheses
            // will have already been parsed and stored in the map.
            println!("unary starting from token {:?} consuming {} tokens", self.tokens[self.pos-rule_result.tokens_consumed], rule_result.tokens_consumed);           
            let token_range_start = self.pos-rule_result.tokens_consumed;
            let token_range_end = self.pos;
            let token = (*self.tokens.get(token_range_start).clone().expect("has value")).clone();
            // ensure that the expression starts with a unary operator.
            match token {
                Token::Minus => {},
                Token::ExclamationMark => {},
                _ => {
                    return rule_result;
                }
            }            
            for token_index in token_range_start..token_range_end {
                // This will skip parentheses
                if !self.token_index_to_node.contains_key(&token_index) {
                    continue;
                }
                // We have located the beginning of the expression inside the parentheses
                let right_hand_side = self.token_index_to_node.get(&token_index).expect("has_value").clone();
                let mut node = new_node_from_token(0, Token::Break);
                match token {
                    Token::Minus => {
                        node = new_node_from_token(token_index, Token::Asterisk);      
                        node.add_child(new_node_from_token(token_index+10001, Token::Num(0)));
                        node.add_child(right_hand_side.1.clone());
                    },
                    Token::ExclamationMark => {
                        node = new_node_from_token(self.pos-rule_result.tokens_consumed, Token::ExclamationMark);
                        node.add_child(right_hand_side.1.clone());
                    },
                    _ => {}
                }
                self.token_index_to_node.insert(
                    token_range_start,
                    (self.pos, node.clone())
                );                    
                break;
            }
        }

        return rule_result;   
    }

    fn base_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Id(String::from("_")))],
            vec![Rhs::Terminal(Token::Num(0))],
            vec![
                Rhs::Terminal(Token::LeftParen),
                Rhs::Nonterminal(Parser::exp_rule),
                Rhs::Terminal(Token::RightParen),
            ]
        ], false);
    }    

    fn precedence_1_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_1_rule),
                Rhs::Nonterminal(Parser::precedence_2_rule),
                Rhs::Nonterminal(Parser::precedence_1_recursive_rule),
            ],
        ], true);
    }

    fn precedence_2_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_2_rule),
                Rhs::Nonterminal(Parser::precedence_3_rule),
                Rhs::Nonterminal(Parser::precedence_2_recursive_rule),
            ],
        ], true);
    }

    fn precedence_3_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_3_rule),
                Rhs::Nonterminal(Parser::precedence_4_rule),
                Rhs::Nonterminal(Parser::precedence_3_recursive_rule),
            ],
        ], true);
    }

    fn precedence_4_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_4_rule),
                Rhs::Nonterminal(Parser::precedence_5_rule),
                Rhs::Nonterminal(Parser::precedence_4_recursive_rule),
            ],
        ], true);
    }

    fn precedence_5_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_5_rule),
                Rhs::Nonterminal(Parser::precedence_6_rule),
                Rhs::Nonterminal(Parser::precedence_5_recursive_rule),
            ],
        ], true);
    }

    fn precedence_6_recursive_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![
                Rhs::Nonterminal(Parser::binop_precedence_6_rule),
                Rhs::Nonterminal(Parser::base_rule),
                Rhs::Nonterminal(Parser::precedence_6_recursive_rule),
            ],
        ], true);
    }

    fn asop_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Assignment)],
            vec![Rhs::Terminal(Token::AdditionAssignment)],
            vec![Rhs::Terminal(Token::SubtractionAssignment)],
            vec![Rhs::Terminal(Token::MultiplicationAssignment)],
            vec![Rhs::Terminal(Token::DivisionAssignment)],
            vec![Rhs::Terminal(Token::ModuloAssignment)]
        ], false)
    }

    fn binop_precedence_1_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Or)],
        ], false)
    }

    fn binop_precedence_2_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::And)],
        ], false)
    }

    fn binop_precedence_3_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Equals)],
            vec![Rhs::Terminal(Token::NotEquals)],
        ], false)
    }

    fn binop_precedence_4_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::GreaterThan)],
            vec![Rhs::Terminal(Token::GreaterThanOrEquals)],
            vec![Rhs::Terminal(Token::LessThan)],
            vec![Rhs::Terminal(Token::LessThanOrEquals)],
        ], false)
    }

    fn binop_precedence_5_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Plus)],
            vec![Rhs::Terminal(Token::Minus)],
        ], false)
    }

    fn binop_precedence_6_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::Asterisk)],
            vec![Rhs::Terminal(Token::Slash)],
            vec![Rhs::Terminal(Token::Percent)],
        ], false)
    }

    fn unop_rule(&mut self) -> RuleResult {
        return self.run_rules_from_rhs(vec![
            vec![Rhs::Terminal(Token::ExclamationMark)],
            vec![Rhs::Terminal(Token::Minus)],
        ], false)
    }

    fn current_token_matches(&mut self, token: &Token) -> bool {
        // Using `mem::discriminant` instead of `==` because rust will compare 
        // both the enum variant AND the data contained in the variant (if applicable)
        // We don't want this behaviour here, since we only care about the enum variant equality
        if mem::discriminant(&self.tokens[self.pos]) == mem::discriminant(token) {
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

    // This method is used to construct the AST nodes for expressions.
    // It takes a token range (start and end indices) that contains
    // an expression and it creates an AST node that models this expression.
    // This node is then inserted in the `token_index_to_node` map.

    // It works in the following way: 
    // There are 2 stacks, one storing operators and one storing operands.
    // An operand is not necessarily a number. It can also be another 
    // expression (in the form of a Node). The code starts by iterating over
    // the given token range. 
    //  * We check if there is an entry in the `token_index_to_node`
    //    map starting from the current token. If we find an entry, it means
    //    that a previous invocation of this method parsed a subexpression
    //    starting from that token, so we skip the tokens starting from
    //    the current one all the way to the `token_end_index` stored in
    //    the map. We push the node stored in the map to the operand stack.
    //  * If the current token is a parenthesis (either opening or closing),
    //    that token is skipped. 
    //  * If the token is an operand (num or variable), we push it to the operand stack. 
    //  * If the token is an operator, we push it to the operator stack.
    // After filling the stacks, we reverse them. We do this because we need the tree
    // to have the 'descending' children on the left of the node. This is important for
    // operands that are not associative. Consider the scenario when parsing  the 
    // expression '1 - 2 - 3'. If we construct a tree with 'descending' child on the
    // right, we end up with '1-(2-3)', which yields a wrong result. By reversing the 
    // stack we prevent this behaviour. We construct the tree in the following way:
    //  1. We pop two operands and one operator from the appropriate stacks and we
    //     create a Node with these three (the parent is the operator, the children are 
    //     the operands).
    //  2. While the operator stack is not empty, we pop one operand and one operator
    //     and, using the node previously created, we create a new node (the parent is 
    //     the operator and the children are the operand and the previous node).
    // After building the AST node, we add it to the `token_index_to_node` map.
    //  
    // NOTE: This method itself is 'dumb', meaning it does not understand operator precedence.
    // All the magic happens when this method is invoked in the grammar's production rules for
    // expressions (fn precedence_x_rule). In a bottom-up apporach, the nodes of the expressions
    // are added to the `token_index_to_node` map based on the precedence of the operators,
    // and then the rule for the lower precedence level will understand that some subexpressions
    // were parsed, and it will ignore them.
    fn construct_expression_node_from_token_range(&mut self, token_range_start: usize, token_range_end: usize) {
        let mut operator_stack = vec![];
        let mut operand_stack = vec![];
        let mut token_index = token_range_start;
        while token_index < token_range_end {
            let token = (*self.tokens.get(token_index).clone().expect("has value")).clone();
            if self.token_index_to_node.contains_key(&token_index) {
                let end_index = self.token_index_to_node.get(&token_index).expect("defined").0;
                operand_stack.push(
                    (
                        token_index, 
                        StackItem::Node(self.token_index_to_node.get(&token_index).expect("defined").1.clone())
                    )
                );
                token_index += end_index - token_index;
                continue;
            }            
            match token {
                Token::LeftParen | Token::RightParen => {
                    token_index += 1;
                    continue;
                },
                Token::Num(_) | Token::Id(_) => {
                    operand_stack.push((token_index, StackItem::Token(token)));
                }
                _ => {
                    operator_stack.push((token_index, token));
                }
            }
            token_index += 1;
        }
        operand_stack.reverse();
        operator_stack.reverse();        
        let mut left_node;
        let first_operand = operand_stack.pop().expect("has value");
        match first_operand.1 {
            StackItem::Token(val) => {
                left_node = new_node_from_token(first_operand.0, val.clone());
            },
            StackItem::Node(val) => {
                left_node = val;
            }
        }
        let mut node = new_node_from_token(0, Token::ExclamationMark);
        if operator_stack.is_empty() {
            node = left_node;
        } else {
            while !operator_stack.is_empty() {
                let operator_node = operator_stack.pop().expect("has value");
                node = new_node_from_token(operator_node.0, operator_node.1);
                let operand_node = operand_stack.pop().expect("has value");
                let right_node;
                match operand_node.1 {
                    StackItem::Token(val) => {
                        right_node = new_node_from_token(operand_node.0, val.clone());
                    },
                    StackItem::Node(val) => {
                        right_node = val;
                    }
                }
                node.add_child(left_node.clone());
                node.add_child(right_node.clone());
                left_node = node.clone();
            }
        }
    
        if !self.token_index_to_node.contains_key(&token_range_start) {
            self.token_index_to_node.insert(
                token_range_start, 
                (token_range_end, node.clone())
            
            );
        } else {
            self.token_index_to_node.get_mut(&token_range_start).expect("not null").0 = token_range_end;
            self.token_index_to_node.get_mut(&token_range_start).expect("not null").1 = node.clone();
        }
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

#[derive(Clone, Debug)]
pub enum StackItem {
    Token(Token),
    Node(Node<usize, Token>)
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

    #[test]
    fn test_expression_simple() {
        let program = String::from(
            "{ \
                var first, second, third : int; \
                first = 1 + 2; \
                second = (first); \
                second += 2; \
                third = first + second / 2; \
                third /= second * (first - second + ((42) + 1)); \
                first = -5; \
                print third; \
                first = (second || third) && second; \
                third = (second >= third) && second || third == 3; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }

    #[test]
    fn test_control_simple() {
        let program = String::from(
            "{ \
                var first, second, third : int; \
                if (a) print 5; \
                if (a == 5) print 5; else print a+4; \
                if (a) { \
                    first = 1 + 2; \
                } \
                if (alpha + 1) { \
                    third = first + second / 2; \
                } else { \
                    third /= second * (first - second + ((42) + 1)); \
                } \
                while (first && third) { \
                    second += 2; \
                    break; \
                } \
                for (a=0; a < third; a+=1) { \
                    third = first + second / 2; \
                    continue; \
                } \
                for (a=0; a < third; a+=1) { \
                    third = first + second / 2; \
                    if (a == 1) { \
                        print a; \
                        continue; \
                    } else { \
                        break; \
                    } \
                } \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }     

    #[test]
    fn test_unary_simple() {
        let program = String::from(
            "{ \
                a = -(1+3); \
                b = !1; \
            }",
        );
        let tokens = get_tokens_from_program(&program);
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.analyze_grammar(), true);
    }
}
