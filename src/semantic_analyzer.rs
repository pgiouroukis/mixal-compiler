use orange_trees::Node;
use crate::lexer::Token;
use std::collections::HashSet;

pub struct SemanticAnalyzer<'a> {
    pub ast: &'a Node<usize, Token>,
    pub symbol_table: HashSet<&'a String>
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(ast: &Node<usize, Token>) -> SemanticAnalyzer {
        SemanticAnalyzer {
            ast,
            symbol_table: HashSet::new()
        }
    }

    pub fn run(&mut self) -> bool {
        let mut violations = 0;
        violations += self.populate_symbol_table_and_check_for_variable_re_declarations();
        violations += self.check_for_undeclared_identifiers();
        return violations == 0;
    }

    fn populate_symbol_table_and_check_for_variable_re_declarations(&mut self) -> u8 {
        let mut violations = 0;
        let variable_declaration_nodes = self.ast.find(&|x| *x.value() == Token::Int);
        for variable_declaration_node in variable_declaration_nodes {
            for identifier_node in variable_declaration_node.children() {
                if let Token::Id(identifier_name) = identifier_node.value() {
                    if self.symbol_table.contains(&identifier_name) {
                        println!("ERROR: re-declaration of identifier '{}'", &identifier_name);
                        violations += 1;
                    } else {
                        self.symbol_table.insert(identifier_name);
                    }
                    
                }
            }
        }
        return violations;
    }

    fn check_for_undeclared_identifiers(&self) -> u8 {
        let violating_nodes = self.ast.find(&|x| {
            if let Token::Id(identifier_name) = x.value() {
                if !self.symbol_table.contains(&identifier_name) {
                    println!("ERROR: undeclared identifier '{}'", identifier_name);
                    return true;
                }
            }
            return false;
        });
        return violating_nodes.len().try_into().unwrap();
    }
}
