use std::sync::{Arc, Mutex};
use either::{Either, Left, Right};
use crate::compiler::parser::{NodeArithmeticExpr, NodeArithmeticOperation, NodeBaseExpr, ParserErrorType, ParserLogger};
use crate::compiler::parser::expression_factory::reverse_polish_notation::ReversePolishNotation;
use crate::compiler::parser::nodes::ResultType;
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::parser::token_stream::TokenStream;

pub struct ExpressionFactory<'a> {
    m_line_stream: &'a mut TokenStream,
    m_logger: Arc<Mutex<ParserLogger>>,
    m_expr_stack: Vec<NodeArithmeticExpr>,
}

impl<'a> ExpressionFactory<'a> {
    pub(crate) fn new(line: &'a mut TokenStream, logger: Arc<Mutex<ParserLogger>>) -> ExpressionFactory<'a> {
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
        if operator_is_bool && !ExpressionFactory::<'a>::type_check_logical_operands(&lhs.clone().unwrap(), &rhs.clone().unwrap())
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
            op: *operator,
            result_type: self.get_result_type(&operator),
        }));
        true
    }
    
    fn type_check_logical_operands(lhs_expr: &NodeArithmeticExpr, rhs_expr: &NodeArithmeticExpr) -> bool{
        let lhs_valid_base = matches!(lhs_expr, NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)));
        let rhs_valid_base = matches!(rhs_expr, NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)));
        let lhs_valid_op = matches!(lhs_expr, NodeArithmeticExpr::Operation(NodeArithmeticOperation{result_type: ResultType::Boolean, ..}));
        let rhs_valid_op = matches!(rhs_expr, NodeArithmeticExpr::Operation(NodeArithmeticOperation{result_type: ResultType::Boolean, ..}));
        (lhs_valid_base || lhs_valid_op) && (rhs_valid_base || rhs_valid_op)
    }

    fn get_result_type(& self, op: &Operator) -> ResultType{
        let mut res = ResultType::Numeric;
        match op{
            Operator::And { .. } |
            Operator::Or { .. } |
            Operator::Xor { .. } |
            Operator::Not { .. } => { res = ResultType::Boolean }
            _ => {}
        }
        res
    }
    
    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.log_error(error, token);
    }
}



#[cfg(test)]
mod test_expression_factory{
    use super::*;
    use crate::compiler::parser::{ParserLogger};
    use crate::compiler::tokenizer::{Token, Operator};
    use std::sync::{Arc, Mutex};
    use crate::compiler::logger::Logger;
    use crate::compiler::span::Span;

    fn setup_logger() -> Arc<Mutex<ParserLogger>> {
        Arc::new(Mutex::new(ParserLogger::new("".to_string(), "".to_string())))
    }

    #[test]
    fn test_base_number_expression() {
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![Token::Number { value: 42.to_string(), span:  dummy_span}], logger.clone());
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(matches!(result, Some(Right(NodeBaseExpr::Num(_)))));
    }

    #[test]
    fn test_base_boolean_expression() {
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![Token::Boolean { value: true , span: dummy_span}], logger.clone());
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(matches!(result, Some(Right(NodeBaseExpr::Bool(_)))));
    }
    
    #[test]
    fn test_empty_input(){
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(Vec::new(), logger.clone());
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(result.is_none());
    }

    #[test]
    fn test_operation() {
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::Operator(Operator::Plus { span: dummy_span }),
            Token::Number { value: 2.to_string(), span: dummy_span }
            
        ],
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(matches!(result, Some(Left(_))));
    }
    
    #[test]
    fn test_bool_operation(){
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::Boolean { value: true, span: dummy_span },
            Token::Operator(Operator::And { span: dummy_span }),
            Token::Boolean { value: false, span: dummy_span },
            Token::Operator(Operator::Or {span: dummy_span}),
            Token::Boolean { value: true, span: dummy_span}
        ],
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(matches!(result, Some(Left(_))));
    }
    
    #[test]
    fn test_right_associative_operation(){
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::Operator(Operator::Exponent { span: dummy_span }),
            Token::Number { value: 2.to_string(), span: dummy_span },
            Token::Operator(Operator::Exponent { span: dummy_span }),
            Token::Number { value: 1.to_string(), span: dummy_span },
        ],
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(matches!(result, Some(Left(_))));
    }

    #[test]
    fn test_missing_operand() {
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::Operator(Operator::Plus { span: dummy_span})],
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(result.is_none());
    }

    #[test]
    fn test_unexpected_token() {
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::ID { name: "x".to_string(), span: dummy_span },
            Token::Number { value: 42.to_string(), span: dummy_span },
            Token::Boolean { value: true, span: dummy_span },
            Token::Err {span: dummy_span}
        ], 
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_wrong_bool_operation(){
        let dummy_span = Span::new(0, 0, 0);
        let logger = setup_logger();
        let mut token_stream = TokenStream::new(vec![
            Token::ID { name: "x".to_string(), span: dummy_span },
            Token::Operator(Operator::And { span: dummy_span }),
            Token::Boolean { value: true, span: dummy_span }
        ],
            logger.clone()
        );
        let mut factory = ExpressionFactory::new(&mut token_stream, logger);

        let result = factory.create();
        assert!(result.is_none());
    }
}