use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

pub struct Tokenizer {

    m_tokens : Vec<Token>,
    m_line: usize,
    m_row: usize
}

impl Tokenizer {

    pub fn new() -> Self {
        Tokenizer { m_tokens: Vec::new(), m_line: 0, m_row: 0}
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
                self.emit_token(Token::Err {span: error_span });
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
        match ch {
            '(' => Some(Token::OpenBracket { span: (self.m_line, (self.m_row, self.m_row)) }),
            ')' => Some(Token::ClosedBracket { span: (self.m_line, (self.m_row, self.m_row)) }),
            '{' => Some(Token::OpenCurlyBracket { span: (self.m_line, (self.m_row, self.m_row)) }),
            '}' => Some(Token::OpenCurlyBracket { span: (self.m_line, (self.m_row, self.m_row)) }),
            '=' => Some(Token::Equals { span: (self.m_line, (self.m_row, self.m_row)) }),
            ' ' => Some(Token::WhiteSpace { span: (self.m_line, (self.m_row, self.m_row)) }),
            '+' => Some(Token::Operator(Operator::Plus { span: (self.m_line, (self.m_row, self.m_row)) })),
            '-' => Some(Token::Operator(Operator::Minus { span: (self.m_line, (self.m_row, self.m_row)) })),
            '%' => Some(Token::Operator(Operator::Modulus { span: (self.m_line, (self.m_row, self.m_row)) })),
            '*' => {
                if peek == Some(&'*'){
                    None
                } else {Some(Token::Operator(Operator::Multiplication { span: (self.m_line, (self.m_row, self.m_row)) }))}
            },
            '\n' => { Some(Token::NewLine { span: (self.m_line, (self.m_row, self.m_row)) })}
            _ => {None}
        }
    }

    fn check_buf(&mut self, buf : &String, input: &Peekable<Chars>) -> Option<Token> {
        match buf.as_str() {
            "exit" => Some(Token::Exit {span : self.get_span(buf.len())}),
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


#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ID { name: String, span: (usize, (usize, usize)) },
    Number { value: String, span: (usize, (usize, usize)) },
    Boolean { value: bool, span: (usize, (usize, usize)) },
    Exit {span: (usize, (usize, usize))},
    OpenBracket {span: (usize, (usize, usize))},
    ClosedBracket {span: (usize, (usize, usize))},
    OpenCurlyBracket {span: (usize, (usize, usize))},
    ClosedCurlyBracket {span: (usize, (usize, usize))},
    Equals {span: (usize, (usize, usize))},
    Operator(Operator),
    WhiteSpace {span: (usize, (usize, usize))},
    NewLine {span: (usize, (usize, usize))},
    Err {span: (usize, (usize, usize))}
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Operator {
    Plus {span: (usize, (usize, usize))},
    Minus {span: (usize, (usize, usize))},
    Multiplication {span: (usize, (usize, usize))},
    Division {span: (usize, (usize, usize))},
    Exponent {span: (usize, (usize, usize))},
    Modulus {span: (usize, (usize, usize))},
    And {span: (usize, (usize, usize))},
    Or {span: (usize, (usize, usize))},
    Xor {span: (usize, (usize, usize))},
    Not {span: (usize, (usize, usize))},
    OpenParenthesis {span: (usize, (usize, usize))},
    ClosedParenthesis {span: (usize, (usize, usize))}
}

impl Token {
    pub fn get_span(&self) -> (usize, (usize, usize)) {
        match self {
            Token::ID { span, .. }
            | Token::Number { span, .. }
            | Token::Boolean { span, .. }
            | Token::Exit { span }
            | Token::OpenBracket { span }
            | Token::ClosedBracket { span }
            | Token::OpenCurlyBracket { span }
            | Token::ClosedCurlyBracket { span }
            | Token::Equals { span }
            | Token::WhiteSpace { span }
            | Token::NewLine { span } => *span,
            Token::Err { span } => { 
                let (line, (start, end)) = *span;
                (line, (start, end+1))
            },
            Token::Operator(op) => op.get_span(),
        }
    }
}

impl Operator {
    pub fn get_span(&self) -> (usize, (usize, usize)) {
        match self {
            Operator::Plus { span }
            | Operator::Minus { span }
            | Operator::Multiplication { span }
            | Operator::Division { span }
            | Operator::Exponent { span }
            | Operator::Modulus { span }
            | Operator::And { span }
            | Operator::Or { span }
            | Operator::Xor { span }
            | Operator::Not { span }
            | Operator::OpenParenthesis { span }
            | Operator::ClosedParenthesis { span } => *span,
        }
    }
}

// Implement Display for Operator
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Operator::Plus { span: _ } => "+",
            Operator::Minus { span: _ } => "-",
            Operator::Multiplication { span: _ } => "*",
            Operator::Division { span: _ } => "/",
            Operator::Exponent { span: _ } => "^",
            Operator::Modulus { span: _ } => "%",
            Operator::And { .. } => "&&",
            Operator::Or { .. } => "||",
            Operator::Xor { .. } => "^|",
            Operator::Not { .. } => "!!",
            Operator::OpenParenthesis { span: _ } => "(",
            Operator::ClosedParenthesis { span: _ } => ")"

        };
        write!(f, "{}", symbol)
    }
}

// Implement Display for Token
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::ID { name, span } => write!(f, "ID({}, {:?})", name, span),
            Token::Number { value, span } => write!(f, "Number({}, {:?})", value, span),
            Token::Exit { .. } => write!(f, "exit()"),
            Token::OpenBracket { .. } => write!(f, "("),
            Token::ClosedBracket { .. } => write!(f, ")"),
            Token::OpenCurlyBracket { .. } => write!(f, "{{"),
            Token::ClosedCurlyBracket { .. } => write!(f, "}}"),
            Token::Equals {..} => write!(f, "="),
            Token::Operator(op) => write!(f, "{}", op),
            Token::WhiteSpace {..} => write!(f, "\\s"),
            Token::NewLine {..} => write!(f, "NewLine"),
            _ => {write!(f, "err")}
        }
    }
}

impl Operator {
    pub fn precedence(self) -> usize {
        match self {
            Operator::Plus { .. } | Operator::Minus { .. } | Operator::And { .. } | Operator::Or { .. } | Operator::Xor { .. } => {0}
            Operator::Multiplication { .. } | Operator::Division { .. } | Operator::Modulus { .. } => {1}
            Operator::OpenParenthesis { .. } | Operator::ClosedParenthesis { .. } | Operator::Exponent { .. } => {2}
            Operator::Not { .. } => 3
        }
    }

    pub fn associativity(self) -> String {
        match self{
            Operator::Exponent { .. } | Operator::Not { .. } => {"Right".to_string()}
            _ => {"Left".to_string()}
        }
    }
}
