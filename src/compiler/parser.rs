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
        while let Some(token) = self.peek(0) {
            if *token == Token::Err{
                return Err("An error was present when parsing".to_string());
            }
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
        let tokens = self.peek_range(2, false).ok_or("Not enough tokens for 'exit' statement".to_string())?;
        // Check if the first token is 'exit'
        if !matches!(tokens.get(0), Some(Token::Exit { .. })) {
            return Err("exit is not present".to_string());
        }
        // Check if the second token is an opening parenthesis
        if !matches!(tokens.get(1), Some(Token::OpenParen)) {
            return Err("Error: Initial '(' is missing".to_string());
        }
        // Advance past 'exit' and '(' tokens
        self.advance(2, false);

        // Parse the arithmetic expression
        let expr = self
            .parse_arithmetic_expr()
            .map_err(|e| format!("Invalid expression in 'exit': {}", e))?;

        // Check for closing parenthesis
        if !matches!(self.peek(0), Some(Token::CloseParen)) {
            return Err("Error: Final ')' is missing.".to_string());
        }

        // Advance past the closing parenthesis
        self.advance(1, false);

        // Return the parsed NodeExit
        match expr{
            Left(operation) => {Ok(NodeExit { expr: NodeArithmeticExpr::Operation(*operation) })}
            Right(base) => {Ok(NodeExit { expr: NodeArithmeticExpr::Base(base) })}
        }
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
                                    Left(operation) => {NodeArithmeticExpr::Operation(*operation)}
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

    fn parse_arithmetic_expr(&mut self) -> Result<Either<Box<NodeArithmeticOperation>, NodeBaseExpr>, String> {
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

                    // Helper function to construct the operation
                    let create_operation = |lhs: NodeArithmeticExpr, rhs: NodeArithmeticExpr| {
                        
                        let lhs_node = match lhs {
                            NodeArithmeticExpr::Base(base) => Right(base),
                            NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
                        };
                        let rhs_node = match rhs {
                            NodeArithmeticExpr::Base(base) => Right(base),
                            NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
                        };
                        NodeArithmeticExpr::Operation(NodeArithmeticOperation {
                            lhs: lhs_node,
                            rhs: rhs_node,
                            op: token,
                        })
                    };

                    let operation_node = create_operation(lhs, rhs);
                    expr_stack.push(operation_node);
                }
                _ => { Err(format!("Unexpected token {token} in arithmetic expression."))?; }
            }
        }
        
        match expr_stack.pop().ok_or("Insufficient operands")?{
            NodeArithmeticExpr::Base(base) => {Ok(Right(base))}
            NodeArithmeticExpr::Operation(op) => {Ok(Left(Box::new(op)))}
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
                        if matches!(operator, Operator::OpenParenthesis) || (operator.precedence() <= op.clone().precedence() && (operator.precedence() != op.clone().precedence() || op.clone().associativity().eq("Right"))){
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
        dbg!(polish.clone());
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

#[derive(Debug, PartialEq)]
pub struct NodeProgram{
    pub(crate) stmts: Vec<NodeStmt>
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeStmt {
    Exit(NodeExit),
    ID(NodeVariableAssignement)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeExit {
    pub(crate) expr: NodeArithmeticExpr
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeVariableAssignement{
    pub variable: Token,
    pub value: NodeArithmeticExpr
}
#[derive(Clone, Debug, PartialEq)]
pub enum NodeArithmeticExpr {
    Base(NodeBaseExpr),
    Operation(NodeArithmeticOperation)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NodeArithmeticOperation {
    pub(crate) lhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>,
    pub(crate) rhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>,
    pub(crate) op: Token
}


#[derive(Clone, Debug, PartialEq)]
pub enum NodeBaseExpr {
    Num(Token),
    ID(Token)
}

impl NodeProgram{
    pub fn get_stmts(& self) -> Vec<NodeStmt>{
        self.stmts.clone()
    }
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
        let lhs_str = if matches!(self.lhs, Left(..)) {
            format!("({})", self.lhs)
        } else{
            format!("{}", self.lhs)
        };
        let rhs_str = if matches!(self.rhs, Left(..)) {
            format!("({})", self.rhs)
        } else{
            format!("{}", self.rhs)
        };
        write!(
            f,
            "{} {} {}",
            lhs_str,
            self.op,
            rhs_str
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
