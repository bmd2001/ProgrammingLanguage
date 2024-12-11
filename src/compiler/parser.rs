use std::collections::VecDeque;
use std::fmt;
use either::{Either, Right, Left};
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
        while let Some(..) = self.peek(0) {
            match self.parse_stmt() {
                Ok(stmt) => {
                    prog.stmts.push(stmt);
                    self.advance(1, true);
                },
                Err(e) => return Err(format!("The Statement number {} wasn't parsed correctly with the following errors:\n{e}", prog.stmts.len()+1))
            }
        }
        dbg!(prog.stmts.len());
        Ok(prog)
    }

    fn parse_stmt(&mut self) -> Result<NodeStmt, String> {
        match self.parse_exit() {
            Ok(exit_node) => Ok(NodeStmt::Exit(exit_node)),
            Err(exit_err) if exit_err != "exit is not present" => {
                // Return an error if parse_exit failed with a critical error
                Err(format!("Failed to parse Exit statement:\n- Exit Error: {exit_err}"))
            },
            _ => {
                match self.parse_variable_assignement() {
                    Ok(var) => Ok(NodeStmt::ID(var)),
                    Err(var_err) => {
                        // Return an error if parse_variable_assignement fails
                        Err(format!(
                            "Failed to parse Variable Assignment statement:\n- Variable Assignment Error: {var_err}"
                        ))
                    }
                }
            }
        }
    }
    
    fn parse_exit(&mut self) -> Result<NodeExit, String>{
        let first_is_exit = matches!(self.peek(0), Some(Token::Exit { span: _ }));
        if !first_is_exit {
            return Err("exit is not present".to_string())
        }
        let second_is_oparen = matches!(self.peek(1), Some(Token::OpenParen));
        if second_is_oparen {
            self.advance(2, false);
            let expr = self.parse_arithmetic_expr().map_err(|e| format!("Invalid expression in 'exit': {}", e))?;
            let last_is_cparen = matches!(self.peek(0), Some(Token::CloseParen));
            return if last_is_cparen {
                self.advance(1, false);
                Ok( match expr {
                    Left(b) => {if let NodeArithmeticExpr::Operation(NodeArithmeticOperation { lhs, rhs, op }) = *b {
                        NodeExit{ expr: NodeArithmeticExpr::Operation(NodeArithmeticOperation {
                            lhs,
                            rhs,
                            op,
                        })}
                    } else {
                        // Handle the unexpected case (optional)
                        panic!("Expected a NodeArithmeticExpr::Operation, found something else.");
                    }}
                    Right(base) => {NodeExit{expr: NodeArithmeticExpr::Base(base)}}
                })
            } else {
                Err("Error: Final ')' is missing.".to_string())
            }
        }
        Err("Error: Initial '(' is missing".to_string())
    }
    
    fn parse_variable_assignement(&mut self) -> Result<NodeVariableAssignement, String>{
        if let Some(tokens) = self.peek_range(3, true){
            return match &tokens[..2] {
                [
                ref id @ Token::ID { .. },           // First token: Identifier
                Token::Equals { .. },            // Second token: Equals
                ] => {
                    self.advance(2, true);
                    match self.parse_arithmetic_expr() {
                        Ok(expr) => {
                            Ok(NodeVariableAssignement {
                                variable: id.clone(),
                                value: {match expr{
                                    Left(b) => {// Dereference the Box to access the NodeArithmeticExpr inside
                                        if let NodeArithmeticExpr::Operation(NodeArithmeticOperation { lhs, rhs, op }) = *b {
                                            NodeArithmeticExpr::Operation(NodeArithmeticOperation {
                                                lhs,
                                                rhs,
                                                op,
                                            })
                                        } else {
                                            // Handle the unexpected case (optional)
                                            panic!("Expected a NodeArithmeticExpr::Operation, found something else.");
                                        }
                                    }
                                    Right(base) => {NodeArithmeticExpr::Base(base)}
                                }
                                },  // The parsed value as a ArithmeticExpr
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

    fn parse_arithmetic_expr(&mut self) -> Result<Either<Box<NodeArithmeticExpr>, NodeBaseExpr>, String> {
        let polish = self.create_reverse_polish_expr().map_err(|e| format!("Failed to create reverse PolishExpr: {}", e))?;
        let mut expr_stack: Vec<NodeArithmeticExpr> = Vec::new();
        for token in polish{
            dbg!(token.clone());
            match token {
                Token::ID { .. } => {
                    expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::ID(token.clone())));
                },
                Token::Number { .. } => {
                    expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::Num(token.clone())));
                },
                Token::Operator(op) => {
                    let rhs = expr_stack.pop().ok_or("Insufficient operands")?;
                    let lhs = expr_stack.pop().ok_or("Insufficient operands")?;
                    if let NodeArithmeticExpr::Base(rhs_base) = rhs{
                        expr_stack.push(NodeArithmeticExpr::Operation(NodeArithmeticOperation{
                            lhs: Left(Box::new(lhs)),
                            rhs: rhs_base,
                            op: Token::Operator(op),
                        }))
                    } else if  let NodeArithmeticExpr::Base(lhs_base) = lhs{
                        expr_stack.push(NodeArithmeticExpr::Operation(NodeArithmeticOperation{
                            lhs: Left(Box::new(rhs)),
                            rhs: lhs_base,
                            op: Token::Operator(op),
                        }))
                    }
                    else {
                        return Err("Parsing went wrong.".to_string())
                    }
                }
                _ => { Err(format!("Unexpected token {token} in arithmetic expression."))?; }
            }
        }
        
        match expr_stack.pop().ok_or("Insufficient operands")?{
            NodeArithmeticExpr::Base(base) => {Ok(Right(base))}
            NodeArithmeticExpr::Operation(op) => {Ok(Left(Box::new(NodeArithmeticExpr::Operation(op))))}
        }
        
    }
    
    fn create_reverse_polish_expr(&mut self) -> Result<VecDeque<Token>, String>{
        let mut stack: Vec<Operator> = Vec::new();
        let mut polish: VecDeque<Token> = VecDeque::new();
        while let Some(token) = self.peek(0) {
            match token{
                Token::ID { .. } | Token::Number { .. } => {
                    polish.push_back(token.clone());
                }
                Token::OpenParen => {
                    stack.push(Operator::OpenParenthesis)
                }
                Token::CloseParen => {
                    if self.peek(1).is_some() && !matches!(self.peek(1), Some(Token::NewLine)) {
                        while !matches!(stack.last(), Some(Operator::OpenParenthesis)) {
                            if stack.is_empty() {
                                Err("Missmatched Parenthesis: ( is missing".to_string())?
                            }
                            let op = stack.pop().unwrap();
                            polish.push_back(Token::Operator(op))
                        }
                        stack.pop();
                    } else {break;}
                }
                Token::Operator(op) => {
                    while let Some(operator) = stack.pop() {
                        if matches!(operator, Operator::OpenParenthesis) || operator.precedence() <= op.clone().precedence(){
                            stack.push(operator);
                            break;
                        }
                        polish.push_back(Token::Operator(operator));
                    }
                    stack.push(op.clone());
                },
                Token::NewLine => {
                    break;
                }
                _ => Err(format!("Unexpected token {token} in arithmetic expression."))?,
            }
            self.advance(1, true);
        }
        while let Some(i) = stack.pop(){
            if matches!(i, Operator::OpenParenthesis){Err("Missmatched Parenthesis: ) is missing")?}
            polish.push_back(Token::Operator(i));
        }
        Ok(polish)
    }

    fn parse_base_expr(&mut self, token: Token) -> Result<NodeBaseExpr, String> {
        match token {
            Token::ID { .. } => {Ok(NodeBaseExpr::ID(token.clone()))}
            Token::Number { .. } => {Ok(NodeBaseExpr::Num(token.clone()))}
            _ => {Err("The parsed token was not an Identifier or a Number".to_string())}
        }
    }
    
    fn peek(& self, offset: usize) -> Option<&Token> {
        if self.m_index + offset >= self.m_tokens.len() {
            return None;
        }
        Some(&self.m_tokens[self.m_index + offset])
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
                if matches!(*token, Token::NewLine){
                    break
                }
                index += 1;
            }
        } else {
            result = self.m_tokens.get(index..index + count)?.to_vec();
        }
        if result.len() == count && !result.contains(&Token::NewLine){
            return Some(result)
        }
        None
    }
    
    fn expect_token(&mut self, token_type: &Token, avoid_space: bool) -> bool {
        if avoid_space {
        }
        if let Some(token) = self.peek(0){
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
    pub(crate) expr: NodeArithmeticExpr
}

#[derive(Clone)]
pub struct NodeVariableAssignement{
    pub(crate) variable: Token,
    pub(crate) value: NodeArithmeticExpr
}
#[derive(Clone)]
pub(crate) enum NodeArithmeticExpr {
    Base(NodeBaseExpr),
    Operation(NodeArithmeticOperation)
}

#[derive(Clone)]
pub(crate) struct NodeArithmeticOperation {
    pub(crate) lhs: Either<Box<NodeArithmeticExpr>, NodeBaseExpr>,
    pub(crate) rhs: NodeBaseExpr,
    pub(crate) op: Token
}


#[derive(Clone)]
pub(crate) enum NodeBaseExpr {
    Num(Token),
    ID(Token)
}


impl fmt::Display for NodeVariableAssignement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Token::ID { name, .. } = &self.variable {
            write!(f, "{} = {}", name, self.value)
        } else {
            write!(f, "Invalid variable token")
        }
    }
}

impl fmt::Display for NodeArithmeticExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeArithmeticExpr::Base(base) => {write!(f, "{base}")}
            NodeArithmeticExpr::Operation(op) => {write!(f, "{op}")}
        }
    }
}

impl fmt::Display for NodeArithmeticOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the operation as `lhs op rhs`
        write!(
            f,
            "{} {} {}",
            self.lhs,           // Left-hand side
            self.op,            // Operator
            self.rhs           // Dereference Box for right-hand side
        )
    }
}

impl fmt::Display for NodeBaseExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeBaseExpr::Num(Token::Number { value, .. }) => write!(f, "{}", value),
            NodeBaseExpr::ID(Token::ID { name, .. }) => write!(f, "{}", name),
            _ => write!(f, "Invalid base expression"),
        }
    }
}
