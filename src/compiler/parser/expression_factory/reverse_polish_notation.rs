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

    fn handle_operators(&mut self, op: Operator) -> bool{
        match op{
            Operator::OpenBracket { .. } => {
                self.m_stack.push(op);
            }
            Operator::ClosedBracket { .. } => {
                while !matches!(self.m_stack.last(), Some(Operator::OpenBracket {..})) {
                    if self.m_stack.is_empty() {
                        self.log_error(ParserErrorType::ErrExpressionOpenBracketMissing, &Token::Operator(op));
                        return false;
                    }
                    let op = self.m_stack.pop().unwrap();
                    self.m_polish.push(Token::Operator(op))
                }
                self.m_stack.pop();
            }
            _ => {
                while let Some(operator) = self.m_stack.pop() {
                    if matches!(operator, Operator::OpenBracket {..}) || (operator.precedence() <= op.clone().precedence() && (operator.precedence() != op.clone().precedence() || op.clone().associativity().eq("Right"))){
                        self.m_stack.push(operator);
                        break;
                    }
                    self.m_polish.push(Token::Operator(operator));
                }
                self.m_stack.push(op);
            }
        }
        true
    }

    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.test(error, token);
    }
}