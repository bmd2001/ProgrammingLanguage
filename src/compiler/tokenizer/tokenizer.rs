use std::iter::Peekable;
use std::str::Chars;
use super::token::Token;
use super::operator::Operator;
use super::parenthesis_handler::ParenthesisHandler;

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

    pub fn get_tokens(&self) -> Vec<Token> {
        self.m_tokens.clone()
    }
    
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
        let buf = &mut String::new();
        
        let mut chars = input.chars().peekable();
        while let Some(ch) = chars.next(){
            buf.push(ch);
            if let Some(token) = self.check_buf(buf, &chars) {
                self.emit_token(token);
                buf.clear();
            }
            else if buf.len() > 1 && ch == '\n'{
                let error_span = self.get_span(buf.len()-1);
                let new_line_span = self.get_span(1);
                self.emit_token(Token::Err {span: error_span });
                self.emit_token(Token::NewLine {span: new_line_span});
                buf.clear();
            }
            else if let Some(token) = self.match_ch(ch, chars.peek()) {
                self.emit_token(token);
                buf.clear();
            }
        }
        if !buf.is_empty() {
            let error_span = self.get_span(buf.len());
            self.emit_token(Token::Err {span: error_span });
        }
    }
    
    fn match_ch(&mut self, ch: char, peek: Option<&char>) -> Option<Token> {
        let span = (self.m_line, (self.m_row, self.m_row));
        match ch {
            '(' | ')' => Some(self.m_parenthesis_handler.emit_bracket_token(span, ch)),
            '{' => Some(Token::OpenCurlyBracket { span }),
            '}' => Some(Token::ClosedCurlyBracket { span }),
            '=' => Some(Token::Equals { span }),
            ' ' => Some(Token::WhiteSpace { span }),
            '+' => Some(Token::Operator(Operator::Plus { span })),
            '-' => Some(Token::Operator(Operator::Minus { span })),
            '%' => Some(Token::Operator(Operator::Modulus { span })),
            '*' => {
                if peek == Some(&'*'){
                    None
                } else {Some(Token::Operator(Operator::Multiplication { span }))}
            },
            '\n' => {
                self.m_parenthesis_handler.deactivate_function_detector();
                Some(Token::NewLine { span })
            }
            _ => {None}
        }
    }

    fn check_buf(&mut self, buf : &String, input: &Peekable<Chars>) -> Option<Token> {
        match buf.as_str() {
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
            _ => {
                if let Some(token) = self.tokenize_primary_expr(buf, input){
                    Some(token)
                } else {None}
            }
        }
    }

    fn tokenize_primary_expr(&mut self, buf : &String, input: &Peekable<Chars>) -> Option<Token> {
        let mut chars = buf.chars();
        let next_char = input.clone().peek().map(|c| *c).unwrap_or(' ');
        
        // Check if the buffer contains only digits and the next character is not a digit
        let is_buf_number = chars.all(char::is_numeric) && !next_char.is_alphanumeric();
        if  is_buf_number{
            return Some(Token::Number {
                value: buf.to_string(),
                span: self.get_span(buf.len()),
            });
        }
        
        // Check if the buffer starts with a letter and all other characters are alphanumeric
        let is_buf_id = buf.chars().next()?.is_alphabetic() && !next_char.is_alphanumeric() && chars.all(char::is_alphanumeric);
        if  is_buf_id{
            return Some(Token::ID {
                name: buf.to_string(),
                span: self.get_span(buf.len()),
            });
        }
        
        // Check if the buffer contains a badly defined ID/Number
        let is_buf_err = chars.all(char::is_alphanumeric) && buf.chars().next()?.is_numeric() && !next_char.is_alphanumeric();
        if is_buf_err{
            return Some(Token::Err {span: self.get_span(buf.len())});
        }
        None
    }
    
    fn handle_newline(&mut self){
        self.m_line += 1;
        self.m_row = 0;
    }
    
    fn get_span(&mut self, length: usize) -> (usize, (usize, usize)){
        let start = self.m_row;
        self.m_row += length - 1;
        (self.m_line, (start, start + length - 1))
    }
}