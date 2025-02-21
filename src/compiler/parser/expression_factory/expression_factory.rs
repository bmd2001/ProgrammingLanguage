use std::sync::{Arc, Mutex};
use either::{Either, Left, Right};
use crate::compiler::parser::{NodeArithmeticExpr, NodeArithmeticOperation, NodeBaseExpr, ParserErrorType, ParserLogger};
use crate::compiler::parser::expression_factory::reverse_polish_notation::ReversePolishNotation;
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::parser::token_stream::TokenStream;

pub struct ExpressionFactory<'a> {
    m_line_stream: &'a mut TokenStream,
    m_logger: Arc<Mutex<ParserLogger>>,
    m_expr_stack: Vec<NodeArithmeticExpr>,
}

impl<'a> ExpressionFactory<'a> {
    pub fn new(line: &'a mut TokenStream, logger: Arc<Mutex<ParserLogger>>) -> ExpressionFactory<'a> {
        ExpressionFactory {
            m_line_stream: line,
            m_logger: logger,
            m_expr_stack: Vec::new(),
        }
    }
    
    pub fn create(&mut self) -> Option<Either<Box<NodeArithmeticOperation>, NodeBaseExpr>>{
        let polish = ReversePolishNotation::new(self.m_line_stream, self.m_logger.clone()).create();
        if let Some(p) = polish{
            for token in p{
                match token {
                    Token::ID { .. } => {
                        self.m_expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::ID(token.clone())));
                    },
                    Token::Number { .. } => {
                        self.m_expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::Num(token.clone())));
                    },
                    Token::Boolean { .. } => {
                        self.m_expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::Bool(token.clone())));
                    },
                    Token::Operator(ref op_token) => {
                        if !self.create_operation(op_token)
                        {
                            return None;
                        }
                    }
                    _ => {
                        self.log_error(ParserErrorType::ErrUnexpectedToken, &token);
                        return None;
                    }
                }
            }
            match self.m_expr_stack.pop(){
                Some(NodeArithmeticExpr::Base(base)) => {Some(Right(base))}
                Some(NodeArithmeticExpr::Operation(op)) => {Some(Left(Box::new(op)))}
                None => {None},
            }
        } else {
            None
        }
    }
    
    fn create_operation(&mut self, operator: &Operator) -> bool{
        let error_token = &Token::Operator(operator.clone());
        let rhs = self.m_expr_stack.pop();
        let lhs = self.m_expr_stack.pop();
        if lhs.is_none() || rhs.is_none() {
            self.log_error(ParserErrorType::ErrMissingOperand, error_token);
            return false;
        }
        // For logical operators, make sure that both operands are booleans.
        let operator_is_bool = matches!(operator, Operator::And { .. } | Operator::Or { .. } | Operator::Xor { .. });
        if operator_is_bool && !ExpressionFactory::<'a>::type_check_logical_operands(lhs.as_ref().unwrap(), rhs.as_ref().unwrap())
        {
            self.log_error(ParserErrorType::ErrTypeMismatch, error_token);
            return false;
        }

        let lhs_node = match lhs.unwrap() {
            NodeArithmeticExpr::Base(base) => Right(base),
            NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
        };
        let rhs_node = match rhs.unwrap() {
            NodeArithmeticExpr::Base(base) => Right(base),
            NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
        };
        self.m_expr_stack.push(NodeArithmeticExpr::Operation(NodeArithmeticOperation {
            lhs: lhs_node,
            rhs: rhs_node,
            op: Token::Operator(operator.clone()),
        }));
        true
    }
    
    fn type_check_logical_operands(lhs_expr: &NodeArithmeticExpr, rhs_expr: &NodeArithmeticExpr) -> bool{
        match (lhs_expr, rhs_expr) {
            (NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)), 
                NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_))) => true,
            _ => false,
        }
    }
    
    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.test(error, token);
    }
}