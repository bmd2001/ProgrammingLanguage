use std::sync::{Arc, Mutex};
use crate::compiler::parser::{ParserErrorType, ParserLogger};
use crate::compiler::parser::token_stream::TokenStream;
use crate::compiler::tokenizer::{Token, Operator};

pub struct ReversePolishNotation<'a>{
    m_line_stream: &'a mut TokenStream,
    m_logger: Arc<Mutex<ParserLogger>>,
    m_stack: Vec<Operator>,
    m_polish: Vec<Token>,
}

impl<'a> ReversePolishNotation<'a>{
    pub fn new(line: &'a mut TokenStream, m_logger: Arc<Mutex<ParserLogger>>) -> ReversePolishNotation{
        ReversePolishNotation{m_line_stream: line, m_logger, m_stack: vec![], m_polish: vec![]}
    }

    pub fn create(&mut self) -> Option<Vec<Token>>{
        while let Some(token) = self.m_line_stream.peek(0) {
            match token{
                Token::ID { .. } | Token::Number { .. } | Token::Boolean { .. } => {
                    self.m_polish.push(token.clone());
                },
                Token::Operator(op) => {
                    if !self.handle_operators(op.clone()){
                        return None;
                    }
                },
                Token::NewLine {..} | Token::ClosedBracket {..} => {
                    break;
                }
                _ => {
                    self.log_error(ParserErrorType::ErrUnexpectedToken, &token);
                },
            }
            self.m_line_stream.advance_skip_tokens(1, true, |token| matches!(token, Some(Token::WhiteSpace {..})));
        }
        while let Some(i) = self.m_stack.pop(){
            if let Operator::OpenBracket { span } = i{
                self.log_error(ParserErrorType::ErrExpressionClosedBracketMissing, &Token::OpenBracket { span });
                return None;
            }
            self.m_polish.push(Token::Operator(i));
        }
        Some(self.m_polish.clone())
    }

    fn handle_operators(&mut self, rhs_op: Operator) -> bool{
        match rhs_op {
            Operator::OpenBracket { .. } => {
                self.m_stack.push(rhs_op);
            }
            Operator::ClosedBracket { .. } => {
                while !matches!(self.m_stack.last(), Some(Operator::OpenBracket {..})) {
                    if self.m_stack.is_empty() {
                        self.log_error(ParserErrorType::ErrExpressionOpenBracketMissing, &Token::Operator(rhs_op));
                        return false;
                    }
                    let op = self.m_stack.pop().unwrap();
                    self.m_polish.push(Token::Operator(op))
                }
                self.m_stack.pop();
            }
            _ => {
                while let Some(lhs_op) = self.m_stack.pop() {
                    let lhs_is_open_bracket = matches!(lhs_op, Operator::OpenBracket {..});
                    let lhs_geq_precedence = lhs_op.precedence() >= rhs_op.precedence();
                    let not_eq_precedence = lhs_op.precedence() != rhs_op.precedence();
                    let rhs_right_associative = rhs_op.associativity().eq("Right");
                    if lhs_is_open_bracket || (lhs_geq_precedence && (not_eq_precedence || rhs_right_associative)){
                        self.m_stack.push(lhs_op);
                        break;
                    }
                    self.m_polish.push(Token::Operator(lhs_op));
                }
                self.m_stack.push(rhs_op);
            }
        }
        true
    }

    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.test(error, token);
    }
}