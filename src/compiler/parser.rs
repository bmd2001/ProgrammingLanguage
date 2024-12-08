use crate::compiler::tokenizer::{Token, Operator};

pub struct Parser {
    m_tokens: Vec<Token>, // Add lifetime annotation
    m_index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { m_tokens: tokens , m_index: 0 }
    }

    pub fn parse(&mut self) -> Result<NodeProgram, String>{
        let mut prog = NodeProgram { stmts: Vec::new() };
        while let Some(..) = self.peek(None) {
            match self.parse_stmt() {
                Ok(stmt) => prog.stmts.push(stmt),
                Err(e) => return Err(format!("The Statement wasn't parsed correctly with the following errors:\n{e}"))
            }
        }
        dbg!(prog.stmts.len());
        Ok(prog)
    }

    fn parse_stmt(&mut self) -> Result<NodeStmt, String> {
        match self.parse_exit() {
            Ok(exit_node) => Ok(NodeStmt::Exit(exit_node)),
            Err(exit_err) => {
                match self.parse_variable_assignement() {
                    Ok(var) => Ok(NodeStmt::ID(var)),
                    Err(var_err) => {
                        Err(format!(
                            "Failed to parse statement:\n- Exit Error: {}\n- Variable Assignment Error: {}",
                            exit_err, var_err
                        ))
                    }
                }
            }
        }
    }
    
    fn parse_exit(&mut self) -> Result<NodeExit, String>{
        let first_is_exit = matches!(self.peek(None), Some(Token::Exit { span: _ }));
        let second_is_oparen = matches!(self.peek(Some(1)), Some(Token::OpenParen));
        if first_is_exit && second_is_oparen {
            self.advance(2, false);
            let expr = self.parse_primary_expr().map_err(|e| format!("Invalid expression in 'exit': {}", e))?;
            let last_is_cparen = matches!(self.peek(None), Some(Token::CloseParen));
            if last_is_cparen {
                self.advance(1, false);
                return Ok(NodeExit { expr });
            } else {
                return Err("Error: Final ')' is missing.".to_string());
            }
        }
        Err("Invalid exit.".to_string())
    }
    
    fn parse_variable_assignement(&mut self) -> Result<NodeVariableAssignement, String>{
        if let Some(tokens) = self.peek_range(3, true){
            return match &tokens[..2] {
                [
                Token::ID { name, span },           // First token: Identifier
                Token::Equals { .. },            // Second token: Equals
                ] => {
                    self.advance(2, true);
                    match self.parse_primary_expr() {
                        Ok(expr) => {
                            Ok(NodeVariableAssignement {
                                variable: Token::ID { name: name.clone(), span: *span },
                                value: expr,  // The parsed value as a PrimaryExpr
                            })
                        }
                        Err(e) => Err(e), // Handle the error if the last token is not a valid PrimaryExpr
                    }
                }
                _ => {
                    Err("Invalid syntax for variable assignment. Expected: 'ID = Number'.".to_string())
                }
            }
        }
        Err("Not enough tokens for variable assignment.".to_string())
    }

    fn parse_primary_expr(&mut self) -> Result<NodePrimaryExpr, String> {
        match self.parse_arithmetic_expr(){
            Ok(expr) => {
                return Ok(NodePrimaryExpr::Arithmetic(
                    expr
                ))
            }
            Err(_) => {
            }
        }
        if let Some(token) = self.peek(None).cloned() {
            return match token {
                Token::Number { .. } | Token::ID { .. } => {
                    self.advance(1, false); // Consume the token
                    Ok(NodePrimaryExpr::Base(
                        token.clone()
                    ))
                }
                _ => Err("Expected a Base Primary Expression (Number or ID).".to_string()),
            }
        }
        Err("No tokens found for Primary Expression.".to_string())
    }

    fn parse_arithmetic_expr(&mut self) -> Result<NodeArithmeticExpr, String> {
        fn match_operand(token: &Token) -> Option<Token> {
            match token {
                Token::Number { value, span } => Some(Token::Number { value: value.clone(), span: *span }),
                Token::ID { name, span } => Some(Token::ID { name: name.clone(), span: *span }),
                _ => None,
            }
        }
        
        if let Some(range) = self.peek_range(3, true){
            return match &range[..3] {
                [
                lhs,
                Token::Operator(op),
                rhs,
                ] => {
                    if let (Some(lhs_token), Some(rhs_token)) = (match_operand(lhs), match_operand(rhs)) {
                        self.advance(3, true);
                        return Ok(NodeArithmeticExpr {
                            lhs: lhs_token,
                            rhs: rhs_token,
                            op: op.clone(),
                        })
                    } else {
                        Err("Invalid Arithmetic Expression.".to_string())
                    }
                }
                _ => Err("Invalid Arithmetic Expression.".to_string())
            }
        }
        Err("No Arithmetic Expression found".to_string())
    }
    
    fn peek(& self, offset: Option<usize>) -> Option<&Token> {
        let off = offset.unwrap_or(0);
        if self.m_index + off >= self.m_tokens.len() {
            return None;
        }
        Some(&self.m_tokens[self.m_index + off])
    }

    fn peek_range(& self, count: usize, avoid_space: bool) -> Option<Vec<Token>> {
        let mut index = self.m_index;
        let mut result = Vec::new();

        if avoid_space {
            while index < self.m_tokens.len() && result.len() < count {
                let token = &self.m_tokens[index];
                if !matches!(*token, Token::WhiteSpace){
                    result.push(token.clone());
                }
                index += 1;
            }
        } else {
            result = self.m_tokens.get(index..index + count)?.to_vec();
        }
        if result.len() == count {
            return Some(result)
        }
        None
    }
    
    fn expect_token(&mut self, token_type: &Token, avoid_space: bool) -> bool {
        if avoid_space {
        }
        if let Some(token) = self.peek(None){
            return matches!(token_type, token);
        }
        false
    }


    fn advance(&mut self, step: usize, inc_space: bool) {
        if !inc_space{
            self.m_index = usize::min(self.m_index + step, self.m_tokens.len());
        }
        else {
            let mut counter = 0;
            let mut offset = 0;

            while self.m_index + offset < self.m_tokens.len() {
                let token = &self.m_tokens[self.m_index + offset];
                if counter < step && !matches!(token, Token::WhiteSpace){
                    counter += 1;
                } else if !matches!(token, Token::WhiteSpace){
                    break;
                }
                offset += 1;
            }
            self.m_index = usize::min(self.m_index + offset, self.m_tokens.len());
        }
    }

}

pub struct NodeProgram{
    pub(crate) stmts: Vec<NodeStmt>
}

#[derive(Clone)]
pub(crate) enum NodeStmt {
    Exit(NodeExit),
    ID(NodeVariableAssignement)
}

#[derive(Clone)]
pub(crate) struct NodeExit {
    pub(crate) expr: NodePrimaryExpr
}

#[derive(Clone)]
pub struct NodeVariableAssignement{
    pub(crate) variable: Token,
    pub(crate) value: NodePrimaryExpr
}

#[derive(Clone)]
pub(crate) enum NodePrimaryExpr {
    Base(Token),
    Arithmetic(NodeArithmeticExpr)
}

#[derive(Clone)]
pub(crate) struct NodeArithmeticExpr {
    pub(crate) lhs: Token,
    pub(crate) rhs: Token,
    pub(crate) op: Operator
}
