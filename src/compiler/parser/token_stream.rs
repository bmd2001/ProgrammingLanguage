use std::sync::{Arc, Mutex};
use crate::compiler::parser::{ParserErrorType};
use crate::compiler::parser::parser_logger::ParserLogger;
use crate::compiler::tokenizer::Token;

pub(crate) struct TokenStream{
    m_tokens: Vec<Vec<Token>>,
    m_logger: Arc<Mutex<ParserLogger>>,
    m_index: usize,
    m_stmt_index: usize
}

impl TokenStream{
    pub fn new(tokens: Vec<Token>, m_logger: Arc<Mutex<ParserLogger>>) -> TokenStream{
        let mut m_tokens = Vec::new();
        let mut line = Vec::new();

        fn trim_whitespace(mut line: Vec<Token>) -> Vec<Token>{
            while let Some(Token::WhiteSpace {..}) = line.first(){
                line.remove(0);
            }
            while let Some(Token::WhiteSpace {..}) = line.last(){
                line.pop();
            }
            line
        }

        for token in tokens{
            match token{
                Token::NewLine { .. } => {
                    if !line.is_empty() {m_tokens.push(trim_whitespace(line))};
                    line = Vec::new();
                }
                Token::OpenCurlyBracket { .. } => {
                    line.push(token);
                    m_tokens.push(trim_whitespace(line));
                    line = Vec::new();
                }
                Token::ClosedCurlyBracket { .. } => {
                    if !line.is_empty(){
                        m_tokens.push(trim_whitespace(line));
                    }
                    m_tokens.push(vec![token]);
                    line = Vec::new();
                }
                _ => line.push(token)
            }
        }
        
        if !line.is_empty(){
            m_tokens.push(trim_whitespace(line));
        }

        TokenStream{ m_tokens, m_logger, m_index: 0, m_stmt_index: 0}
    }

    pub fn peek(&self, step: usize) -> Option<Token>{
        self.m_tokens.get(self.m_stmt_index)?.get(self.m_index+step).cloned()
    }
    
    pub fn peek_back(&self, step: usize) -> Option<Token>{
        self.m_tokens.get(self.m_stmt_index)?.get(self.m_index-step).cloned()
    }

    pub fn peek_range(& self, count: usize, avoid_space: bool) -> Option<Vec<Token>> {
        let mut step = 0;
        let mut result: Vec<Token> = Vec::new();

        if avoid_space {
            while self.peek(step).is_some() && result.len() < count {
                let token = self.peek(step).unwrap();
                if !matches!(token, Token::WhiteSpace {..}){
                    result.push(token);
                }
                step += 1;
            }
        } else {
            result = self.m_tokens.get(self.m_stmt_index)?.get(step..step + count)?.to_vec();
        }
        if result.len() == count{
            return Some(result)
        }
        None
    }

    /*
    pub fn is_empty(&self) -> bool{
        self.m_stmt_index >= self.m_tokens.len() &&
            self.m_tokens.last().map_or(true,
                                        |line| self.m_index >= line.len())
    }
    */

    pub fn is_end(&self) -> bool{
        self.m_stmt_index+1 >= self.m_tokens.len() &&
            self.m_tokens.last().map_or(true,
                                        |line| self.m_index >= line.len())
    }
    
    pub fn is_err_token_present(&self) -> bool{
        let mut offset = 0;
        while self.peek(offset).is_some(){
            if matches!(self.peek(offset).unwrap(), Token::Err { .. }){
                return true;
            }
            offset += 1;
        }
        false
    }

    // Advance Methods
    pub fn advance(&mut self, step: usize){
        let line_len = self.m_tokens[self.m_stmt_index].len();
        self.m_index = usize::min(self.m_index + step, line_len);
    }

    fn advance_skip_head<F>(&mut self, skipping_predicate: F)
    where F: Fn(Option<Token>) -> bool
    {
        let mut step = 0;
        while skipping_predicate(self.peek(step)) {
            step += 1;
        }
        self.advance(step);
    }

    pub fn advance_skip_tokens<F>(&mut self, step: usize, skip_tail: bool, skipping_predicate: F)
    where F: Fn(Option<Token>) -> bool + Copy
    {
        let mut num_valid_chars_passed = 0;
        self.advance_skip_head(skipping_predicate);
        while num_valid_chars_passed != step && self.peek(0).is_some(){
            if !skipping_predicate(self.peek(0)){
                num_valid_chars_passed += 1;
            }
            self.m_index += 1;
        }
        if skip_tail{
            self.advance_skip_head(skipping_predicate);
        }
    }

    pub fn advance_stmt(&mut self, report: bool){
        while self.peek(0).is_some() && report{
            if let Ok(mut logger) = self.m_logger.lock() {
                logger.log_error(ParserErrorType::ErrUnexpectedToken, &self.peek(0).unwrap());
            }
            self.advance(1);
        }
        while self.peek(0).is_none() && !self.is_end(){
            self.m_index = 0;
            self.m_stmt_index += 1;
        }
    }
}