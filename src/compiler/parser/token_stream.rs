use crate::compiler::parser::parser_logger::ParserErrorType;
use crate::compiler::tokenizer::Token;

pub(crate) struct TokenStream{
    m_tokens: Vec<Vec<Token>>,
    m_index: usize,
    m_stmt_index: usize
}

impl TokenStream{
    pub fn new(tokens: Vec<Token>) -> TokenStream{
        let mut m_tokens = Vec::new();
        let mut line = Vec::new();
        
        fn trim_whitespace(mut line: Vec<Token>) -> Vec<Token>{
            if let Some(Token::WhiteSpace {..}) = line.first(){
                line.remove(0);
            }
            if let Some(Token::WhiteSpace {..}) = line.last(){
                line.pop();
            }
            line
        }
        
        for token in tokens{
            match token{
                Token::NewLine { .. } => {
                    m_tokens.push(trim_whitespace(line));
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
        
        TokenStream{ m_tokens, m_index: 0, m_stmt_index: 0}
    }

    pub fn peek(&self, step: usize) -> Option<Token>{
        self.m_tokens.get(self.m_stmt_index)?.get(self.m_index+step).cloned()
    }

    pub fn is_empty(&self) -> bool{
        self.m_stmt_index >= self.m_tokens.len() &&
            self.m_tokens.last().map_or(true, 
                                        |line| self.m_index >= line.len())
    }
    
    pub fn is_end(&self) -> bool{
        self.m_stmt_index >= self.m_tokens.len() && 
            self.m_tokens.last().map_or(true, 
                                        |line| self.m_index >= line.len())
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
    
    pub fn advance_stmt(&mut self, report: bool){
        while self.peek(0).is_some() && report{
            self.report_error(ParserErrorType::ErrUnexpectedToken, None);
            self.advance(1);
        }
        while self.peek(0).is_none() && !self.is_end(){
            self.m_index = 0;
            self.m_stmt_index += 1;
        }
    }
}