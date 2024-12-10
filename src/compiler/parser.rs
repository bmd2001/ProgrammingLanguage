use std::fmt;
use either::{Either, Right, Left};
use crate::compiler::tokenizer::{Token};

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
            let expr = self.parse_arithmetic_expr(0).map_err(|e| format!("Invalid expression in 'exit': {}", e))?;
            let last_is_cparen = matches!(self.peek(None), Some(Token::CloseParen));
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
                    match self.parse_arithmetic_expr(0) {
                        Ok(expr) => {
                            Ok(NodeVariableAssignement {
                                variable: Token::ID { name: name.clone(), span: *span },
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

    fn parse_arithmetic_expr(&mut self, min_prec: usize) -> Result<Either<Box<NodeArithmeticExpr>, NodeBaseExpr>, String> {
        let mut a: Either<Box<NodeArithmeticExpr>, NodeBaseExpr> = Right(self.parse_base_expr().map_err(|e| format!("Invalid arithmetic expression: {}", e))?);
        self.advance(1, true);
        loop {
            let mut precedence = 0;
            let op = if let Some(Token::Operator(op)) = self.peek(None) {
                precedence = op.clone().precedence();
                if precedence < min_prec {
                    break; // Stop parsing if the operator has lower precedence.
                }
                op.clone() // Clone the operator to avoid borrowing `self`.
            } else {
                break; // Exit the loop if no operator is found.
            };
            self.advance(1, true);
            let b = self.parse_arithmetic_expr(precedence+1).map_err(|e| format!("Invalid arithmetic expression: {}", e))?;
            match b {
                Left(b) => {
                    match a.clone() {
                        Left(prev_a) => {a = self.insert_node(prev_a, b).map_err(|e| format!("Recursive algorithm failed: {}", e))?}
                        Right(prev_a) => {a = Left(Box::new(NodeArithmeticExpr::Operation(NodeArithmeticOperation {lhs: Left(b), rhs: prev_a, op: Token::Operator(op.clone())})))}
                    }
                }
                Right(base) => { a = Left(Box::new(NodeArithmeticExpr::Operation(NodeArithmeticOperation{lhs: a, rhs: base, op: Token::Operator(op.clone())})))}
            };
        }
        Ok(a)
    }
    
    fn insert_node(&mut self, a : Box<NodeArithmeticExpr>, b: Box<NodeArithmeticExpr>) -> Result<Either<Box<NodeArithmeticExpr>, NodeBaseExpr>, String>{
        let mut op = match *a {
            NodeArithmeticExpr::Operation(ref op) => op.clone(),
            _ => return Err("Invalid Node: Expected an Operation".to_string()),
        };

        // Traverse left-hand side to find the left-most operation
        while let Left(ref l) = op.lhs {
            if let NodeArithmeticExpr::Operation(ref nested_op) = **l {
                op = nested_op.clone();
            } else {
                return Err("Invalid Node: Expected a nested Operation".to_string());
            }
        }

        // Extract base `lhs` and operator
        let operator = op.op.clone();
        let lhs = match op.lhs {
            Right(base) => base,
            _ => return Err("Invalid Node: Expected a base expression".to_string()),
        };
        op.lhs = Left(Box::new(NodeArithmeticExpr::Operation(NodeArithmeticOperation{
            lhs: Left(b),
            rhs: lhs,
            op: operator,
        })));
        Ok(Left(a))
    }

    fn parse_base_expr(&mut self) -> Result<NodeBaseExpr, String> {
        if let Some(token) = self.peek(None) {
            return match token {
                Token::ID { .. } => {Ok(NodeBaseExpr::ID(token.clone()))}
                Token::Number { .. } => {Ok(NodeBaseExpr::Num(token.clone()))}
                _ => {Err("The parsed token was not an Identifier or a Number".to_string())}
            }
        }
        Err("No more tokens".to_string())
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
