use std::fmt;

pub struct Tokenizer {

    m_tokens : Vec<Token>,
    m_index: usize,
    m_line: usize,
    m_visited: usize
}

impl Tokenizer {

    pub fn new() -> Self {
        Tokenizer { m_tokens: Vec::new() , m_index: 0, m_line: 0, m_visited: 0}
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.m_tokens.clone()
    }

    fn clear(&mut self){
        self.m_tokens = Vec::new();
        self.m_index = 0;
        self.m_line = 0;
        self.m_visited = 0
    }
    pub fn tokenize(&mut self, input: &str){
        dbg!(&input);
        self.clear();
        let mut buf : Vec<char> = Vec::new();
        let mut peek = self.peek(input, None);
        while peek.is_some() {
            let last_char = peek.unwrap();
            buf.push(last_char);
            let token = self.check_buf(&buf, input);
            self.m_index += 1;
            self.m_visited += 1;
            if token.is_some() {
                buf.clear();
                self.m_tokens.push(token.unwrap());
            };
            peek = self.peek(input, None);
        }
        dbg!(self.m_tokens.len());
    }

    fn check_buf(&mut self, buf : &Vec<char>, input: &str) -> Option<Token> {
        let string_buf: String = buf.iter().collect();
        match string_buf.as_str() {
            "exit" => Some(Token::Exit {span : (self.m_line, self.m_visited - 3)}),
            "(" => Some(Token::OpenParen),
            ")" => Some(Token::CloseParen),
            "=" => Some(Token::Equals),
            " " => Some(Token::WhiteSpace),
            "+" => Some(Token::Operator(Operator::Plus)),
            "-" => Some(Token::Operator(Operator::Minus)),
            "*" => Some(Token::Operator(Operator::Multiplication)),
            "//" => Some(Token::Operator(Operator::Division)),
            "\n" => {
                self.m_line += 1;
                self.m_visited = 0;
                Some(Token::NewLine)
            }
            _ => {
                if let Some(token) = self.tokenize_primary_expr(string_buf.as_str(), input) {
                    Some(token)
                } else {
                    None
                }
            }
        }
    }

    fn tokenize_primary_expr(&mut self, buf : &str, input: &str) -> Option<Token> {
        // Check if the buffer contains only digits and the next character is not a digit
        let next_char = self.peek(input, Some(1)).unwrap_or(' ');
        if buf.chars().all(|c| c.is_digit(10)) && !next_char.is_digit(10) && !next_char.is_alphabetic(){
            return Some(Token::Number {value : String::from(buf), span: (self.m_line, self.m_visited + 1 - buf.len()) })
        }
        // Check if the first character is alphabetical and all others are alphanumerical, while also checking the next character is a space
        else if let Some(first_char) = buf.chars().next() {
            if first_char.is_alphabetic() && buf.chars().all(|c| c.is_alphanumeric()) && !next_char.is_alphanumeric() {
                // Return a token for this case (adjust as needed)
                return Some(Token::ID {
                    name: String::from(buf),
                    span: (self.m_line, self.m_visited + 1 - buf.len())
                })
            }
        }
        None
    }

    fn peek(&mut self, input: &str, offset: Option<usize>) -> Option<char> {
        let off = offset.unwrap_or(0);
        if self.m_index + off >= input.len() {
            return None;
        }
        let c = input.chars().nth(self.m_index+off).unwrap(); // Accessing nth char
        Some(c)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ID { name: String, span: (usize, usize) },
    Number { value: String, span: (usize, usize) },
    Exit {span: (usize, usize)},
    OpenParen,
    CloseParen,
    Equals,
    Operator(Operator),
    WhiteSpace,
    NewLine
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiplication,
    Division
}

// Implement Display for Operator
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Multiplication => "*",
            Operator::Division => "/",
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
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::Equals => write!(f, "="),
            Token::Operator(op) => write!(f, "{}", op),
            Token::WhiteSpace => write!(f, "\\s"),
            Token::NewLine => write!(f, "NewLine"),
        }
    }
}

impl Operator {
    pub fn precedence(self) -> usize {
        match self {
            Operator::Plus | Operator::Minus => {0}
            Operator::Multiplication | Operator::Division => {1}
        }
    }
}