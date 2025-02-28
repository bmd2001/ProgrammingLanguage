use std::sync::{Arc, Mutex};
use crate::compiler::parser::{ParserErrorType};
use crate::compiler::parser::parser_logger::ParserLogger;
use crate::compiler::tokenizer::Token;

#[derive(Clone)]
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
                    if !trim_whitespace(line.clone()).is_empty() {m_tokens.push(trim_whitespace(line))};
                    line = Vec::new();
                }
                Token::OpenCurlyBracket { .. } => {
                    if !trim_whitespace(line.clone()).is_empty(){
                        m_tokens.push(trim_whitespace(line));
                    }
                    m_tokens.push(vec![token]);
                    line = Vec::new();
                }
                Token::ClosedCurlyBracket { .. } => {
                    if !trim_whitespace(line.clone()).is_empty(){
                        m_tokens.push(trim_whitespace(line));
                    }
                    m_tokens.push(vec![token]);
                    line = Vec::new();
                }
                _ => line.push(token)
            }
        }
        
        if !trim_whitespace(line.clone()).is_empty(){
            m_tokens.push(trim_whitespace(line));
        }

        TokenStream{ m_tokens, m_logger, m_index: 0, m_stmt_index: 0}
    }

    pub fn peek(&self, step: usize) -> Option<Token>{
        self.m_tokens.get(self.m_stmt_index)?.get(self.m_index+step).cloned()
    }
    
    pub fn peek_back(&self, step: usize) -> Option<Token>{
        if step > self.m_index { return None };
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
    
    pub fn is_err_in_stmt(&self) -> bool{
        let mut offset = 0;
        while self.peek(offset).is_some(){
            if matches!(self.peek(offset), Some(Token::Err { .. })){
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
        while self.peek(0).is_some(){
            if report{
                if let Ok(mut logger) = self.m_logger.lock() {
                    logger.log_error(ParserErrorType::ErrUnexpectedToken, &self.peek(0).unwrap());
                }
            }
            self.advance(1);
        }
        while self.peek(0).is_none() && !self.is_end(){
            self.m_index = 0;
            self.m_stmt_index += 1;
        }
    }
}



#[cfg(test)]
mod test_token_stream{
    use crate::compiler::logger::Logger;
    use crate::compiler::span::Span;
    use super::*;
    
    fn create_stream(tokens: Vec<Token>) -> TokenStream{
        let logger = Arc::new(Mutex::new(ParserLogger::new("".to_string(), "".to_string())));
        TokenStream::new(tokens, logger)
    }
    
    #[test]
    fn test_init(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::ID { name: "x".to_string(), span: dummy_span },
            Token::Equals { span: dummy_span },
            Token::WhiteSpace{ span: dummy_span },
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::NewLine { span: dummy_span },
            Token::WhiteSpace{ span: dummy_span },
            Token::ID { name: "x".to_string(), span: dummy_span },
            Token::Equals { span: dummy_span },
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::WhiteSpace{ span: dummy_span },
            Token::OpenCurlyBracket {span: dummy_span},
            Token::Equals {span: dummy_span},
            Token::ClosedCurlyBracket {span: dummy_span},
            Token::WhiteSpace {span: dummy_span}
        ];
        
        let token_stream = create_stream(tokens);
        let expected_stream_tokens: Vec<Vec<Token>> = vec![
            vec![
                Token::ID { name: "x".to_string(), span: dummy_span },
                Token::Equals { span: dummy_span },
                Token::WhiteSpace{ span: dummy_span },
                Token::Number { value: 1.to_string(), span: dummy_span },
            ],
            vec![
                Token::ID { name: "x".to_string(), span: dummy_span },
                Token::Equals { span: dummy_span },
                Token::Number { value: 1.to_string(), span: dummy_span },
            ],
            vec![
                Token::OpenCurlyBracket {span: dummy_span},
            ],
            vec![
                Token::Equals {span: dummy_span},
            ],
            vec![
                Token::ClosedCurlyBracket {span: dummy_span}
            ]
        ];
        assert_eq!(token_stream.m_tokens, expected_stream_tokens);
        assert_eq!(token_stream.m_index, 0);
        assert_eq!(token_stream.m_stmt_index, 0);
    }
    
    #[test]
    fn test_peek(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![Token::ClosedBracket {span:dummy_span}];
        let token_stream = create_stream(tokens);
        assert_eq!(token_stream.peek(0), Some(Token::ClosedBracket {span:dummy_span}));
        assert!(token_stream.peek(1).is_none());
    }
    
    #[test]
    fn test_peek_back(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket{span:dummy_span},
            Token::ClosedBracket {span:dummy_span}
        ];
        let mut token_stream = create_stream(tokens);
        token_stream.advance(1);
        assert_eq!(token_stream.peek_back(1), Some(Token::OpenBracket {span:dummy_span}));
        assert!(token_stream.peek_back(2).is_none());
    }
    
    #[test]
    fn test_peek_range(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket{span:dummy_span},
            Token::WhiteSpace{span:dummy_span},
            Token::ClosedBracket {span:dummy_span}
        ];
        let token_stream = create_stream(tokens);
        
        let exp_tokens = vec![
            Token::OpenBracket{span:dummy_span},
            Token::ClosedBracket {span:dummy_span}
        ];
        assert_eq!(token_stream.peek_range(2, true), Some(exp_tokens));
        
        let exp_tokens_space = vec![
            Token::OpenBracket{span:dummy_span},
            Token::WhiteSpace{span:dummy_span}
        ];
        assert_eq!(token_stream.peek_range(2, false), Some(exp_tokens_space));
        
        assert!(token_stream.peek_range(3, true).is_none());
        assert!(token_stream.peek_range(4, false).is_none());
    }
    
    #[test]
    fn test_is_end(){
        assert!(create_stream(Vec::new()).is_end());

        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket {span:dummy_span},
            Token::NewLine {span:dummy_span},
            Token::ClosedBracket {span:dummy_span}
        ];
        let mut token_stream = create_stream(tokens);
        assert!(!token_stream.is_end());
        token_stream.advance_stmt(false);
        assert!(!token_stream.is_end());
        token_stream.advance(1);
        assert!(token_stream.is_end())
    }
    
    #[test]
    fn test_err_in_stmt(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket {span:dummy_span},
            Token::NewLine {span:dummy_span},
            Token::ClosedBracket {span:dummy_span},
            Token::Err {span:dummy_span}
        ];
        let mut token_stream = create_stream(tokens);
        assert!(!token_stream.is_err_in_stmt());
        token_stream.advance_stmt(false);
        assert!(token_stream.is_err_in_stmt())
    }
    
    #[test]
    fn test_advance(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket{span:dummy_span},
            Token::WhiteSpace{span:dummy_span},
            Token::ClosedBracket {span:dummy_span},
            Token::NewLine {span:dummy_span},
            Token::ClosedBracket {span:dummy_span},
        ];
        let mut token_stream = create_stream(tokens);
        
        token_stream.advance(0);
        assert_eq!(token_stream.m_index, 0);
        assert_eq!(token_stream.m_stmt_index, 0);
        
        token_stream.advance(1);
        assert_eq!(token_stream.m_index, 1);
        assert_eq!(token_stream.m_stmt_index, 0);

        token_stream.advance(100);
        assert_eq!(token_stream.m_index, 3);
        assert_eq!(token_stream.m_stmt_index, 0);
    }
    
    #[test]
    fn test_advance_skip_head(){
        let dummy_span = Span::new(0, 0, 0);
        let mut tokens = Vec::new();
        for _ in 0..10{
            tokens.push(Token::OpenBracket {span:dummy_span});
        }
        tokens.push(Token::ClosedBracket {span:dummy_span});
        tokens.push(Token::OpenBracket {span:dummy_span});
        let mut token_stream = create_stream(tokens);
        
        let predicate = |token| matches!(token, Some(Token::OpenBracket {..}));
        token_stream.advance_skip_head(predicate);
        assert_eq!(token_stream.m_index, 10);
        assert_eq!(token_stream.peek(0), Some(Token::ClosedBracket {span:dummy_span}));
    }
    
    #[test]
    fn test_advance_skip_token(){
        let dummy_span = Span::new(0, 0, 0);
        let mut tokens = Vec::new();
        for _ in 0..10{
            tokens.push(Token::OpenBracket {span:dummy_span});
        }
        tokens.push(Token::ClosedBracket {span:dummy_span});
        tokens.push(Token::OpenBracket {span:dummy_span});
        let mut token_stream = create_stream(tokens);

        let predicate = |token| matches!(token, Some(Token::OpenBracket {..}));
        token_stream.advance_skip_tokens(0, false, predicate);
        assert_eq!(token_stream.m_index, 10);
        assert_eq!(token_stream.peek(0), Some(Token::ClosedBracket {span:dummy_span}));
        
        token_stream.m_index = 0;
        token_stream.advance_skip_tokens(1, true, predicate);
        assert_eq!(token_stream.m_index, 12);
        assert!(token_stream.peek(0).is_none());
    }
    
    #[test]
    fn test_advance_stmt(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::OpenBracket {span:dummy_span},
            Token::NewLine {span:dummy_span},
            Token::ClosedBracket {span:dummy_span},
        ];
        let mut token_stream = create_stream(tokens);
        
        token_stream.advance_stmt(false);
        assert_eq!(token_stream.m_index, 0);
        assert_eq!(token_stream.m_stmt_index, 1);
        
        token_stream.advance_stmt(false);
        assert_eq!(token_stream.m_index, 1);
        assert_eq!(token_stream.m_stmt_index, 1);
        assert!(token_stream.peek(0).is_none());
        assert!(token_stream.is_end());
    }
}