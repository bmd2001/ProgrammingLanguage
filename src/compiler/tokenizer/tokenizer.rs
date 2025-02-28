use super::operator::Operator;
use super::parenthesis_handler::ParenthesisHandler;
use super::token::Token;
use crate::compiler::span::Span;
use std::iter::Peekable;
use std::str::Chars;

pub struct Tokenizer {
    m_tokens : Vec<Token>,
    m_line: usize,
    m_row: usize,
    m_parenthesis_handler: ParenthesisHandler,
}

impl Tokenizer {

    pub fn new() -> Self {
        Tokenizer { m_tokens: Vec::new(), m_line: 0, m_row: 0, m_parenthesis_handler: ParenthesisHandler::new()}
    }

    pub fn get_tokens(&self) -> Vec<Token> { self.m_tokens.clone() }
    
    fn emit_token(&mut self, token : Token) {
        self.m_row += 1;
        if matches!(token, Token::NewLine {..}){
            self.handle_newline();
        }
        self.m_tokens.push(token);
    }

    fn clear(&mut self){
        self.m_tokens.clear();
        self.m_line = 0;
        self.m_row = 0
    }
    
    pub fn tokenize(&mut self, input: &str){
        self.clear();
        let mut buf = String::new();
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next(){
            buf.push(ch);
            if let Some(token) = self.check_buf(&buf, &mut chars) {
                self.emit_token(token);
                buf.clear();
                continue;
            }
            if buf.len() > 1 && ch == '\n'{
                let error_span = self.get_span(buf.len()-1);
                let new_line_span = self.get_span(1);
                self.emit_token(Token::Err {span: error_span });
                self.emit_token(Token::NewLine {span: new_line_span});
                buf.clear();
                continue;
            }
            if let Some(token) = self.match_ch(ch, chars.peek()) {
                self.emit_token(token);
                buf.clear();
                continue;
            }
        }
        if !buf.is_empty() {
            let error_span = self.get_span(buf.len());
            self.emit_token(Token::Err {span: error_span });
        }
    }
    
    fn match_ch(&mut self, ch: char, peek: Option<&char>) -> Option<Token> {
        let span = Span::new(self.m_line, self.m_row, self.m_row);
        match ch {
            '(' | ')' => Some(self.m_parenthesis_handler.emit_bracket_token(span, ch == '(')),
            '{' => Some(Token::OpenCurlyBracket { span }),
            '}' => Some(Token::ClosedCurlyBracket { span }),
            '=' => Some(Token::Equals { span }),
            '+' => Some(Token::Operator(Operator::Plus { span })),
            '-' => Some(Token::Operator(Operator::Minus { span })),
            '%' => Some(Token::Operator(Operator::Modulus { span })),
            '*' => {
                if peek == Some(&'*'){
                    None
                } else {Some(Token::Operator(Operator::Multiplication { span }))}
            },
            ' ' => {
                if peek != Some(&' ') {
                    return Some(Token::WhiteSpace { span })
                }
                None
            },
            '\n' => {
                self.m_parenthesis_handler.deactivate_function_detector();
                Some(Token::NewLine { span })
            }
            _ => {None}
        }
    }

    fn check_buf(&mut self, buf : &str, input: &mut Peekable<Chars>) -> Option<Token> {
        match buf.replace(" ", "").as_str(){
            "exit" => {
                self.m_parenthesis_handler.activate_function_detector();
                Some(Token::Exit {span : self.get_span(buf.len())})
            },
            "**" => Some(Token::Operator(Operator::Exponent {span : self.get_span(buf.len())})),
            "//" => Some(Token::Operator(Operator::Division {span : self.get_span(buf.len())})),
            "&&" => Some(Token::Operator(Operator::And { span: self.get_span(buf.len()) })),
            "||" => Some(Token::Operator(Operator::Or { span: self.get_span(buf.len()) })),
            "!!" => Some(Token::Operator(Operator::Not { span: self.get_span(buf.len()) })),
            "^|" => Some(Token::Operator(Operator::Xor { span: self.get_span(buf.len()) })),
            "true" => Some(Token::Boolean { value: true, span: self.get_span(buf.len()) }),
            "false" => Some(Token::Boolean { value: false, span: self.get_span(buf.len()) }),
            "" => {
                if let Some(' ') = input.peek(){
                    return None
                }
                Some(Token::WhiteSpace {span: self.get_span(buf.len())})
            }
            _ => {
                if let Some(token) = self.tokenize_primary_expr(buf, input){
                    Some(token)
                } else {None}
            }
        }
    }

    fn tokenize_primary_expr(&mut self, buf : &str, input: &mut Peekable<Chars>) -> Option<Token> {
        self.get_num(buf, input)
            .or_else(|| self.get_id(buf, input))
            .or_else(|| self.get_error(buf, input))
    }
    
    fn get_num(&mut self, buf: &str, input: &mut Peekable<Chars>) -> Option<Token>{
        let next_char = input.peek().unwrap_or(&' ');
        if buf.chars().all(char::is_numeric) && !next_char.is_alphanumeric() {
            let value = buf.to_string();
            let value_len = buf.len();
            return Some(Token::Number {
                value,
                span: self.get_span(value_len),
            });
        }
        None
    }
    
    fn get_id(&mut self, buf: &str, input: &mut Peekable<Chars>) -> Option<Token>{
        let next_char = input.peek().unwrap_or(&' ');
        let mut chars = buf.chars();
        // Ensure there's at least one character and it is alphabetic.
        if let Some(first) = chars.next() {
            // Check that the entire buffer is alphanumeric and the next char is not part of an identifier.
            if first.is_alphabetic()
                && buf.chars().all(char::is_alphanumeric)
                && !next_char.is_alphanumeric()
            {
                let value = buf.to_string();
                let value_len = buf.len();
                return Some(Token::ID {
                    name: value,
                    span: self.get_span(value_len),
                });
            }
        }
        None
    }
    
    fn get_error(&mut self, buf: &str, input: &mut Peekable<Chars>) -> Option<Token>{
        let next_char = input.peek().unwrap_or(&' ');
        let mut chars = buf.chars();
        if let Some(first) = chars.next() {
            // If the buffer starts with a numeric but then contains only alphanumerics,
            // it may be an error if it doesn't form a proper number.
            if first.is_numeric()
                && buf.chars().all(char::is_alphanumeric)
                && !next_char.is_alphanumeric()
            {
                let value_len = buf.len();
                return Some(Token::Err {
                    span: self.get_span(value_len),
                });
            }
        }
        None
    }
    
    fn handle_newline(&mut self){
        self.m_line += 1;
        self.m_row = 0;
    }
    
    fn get_span(&mut self, length: usize) -> Span {
        let res = Span::new(self.m_line, self.m_row, self.m_row + length-1);
        self.m_row += length - 1;
        res
    }
}