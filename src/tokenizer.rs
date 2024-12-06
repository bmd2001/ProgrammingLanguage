use std::cmp::PartialEq;

pub struct Tokenizer {
    m_tokens : Vec<Token>,
    m_index: usize
}

impl Tokenizer {

    pub fn new() -> Self {
        Tokenizer { m_tokens: Vec::new() , m_index:0 }
    }
    pub fn tokenize(&mut self, input: &str){
        dbg!(&input);
        let mut buf : Vec<char> = Vec::new();
        let mut peek = self.peek(input);
        while peek.is_some() {
            let last_char = peek.unwrap();
            if last_char.is_alphanumeric() || buf.is_empty(){
                buf.push(last_char);
            }
            let token = self.check_buf(&buf);
            if token.is_some() {
                self.m_tokens.push(token.unwrap());
                buf.clear();
            };
            self.m_index += 1;
            peek = self.peek(input);
        }
        dbg!(self.m_tokens.len());
    }

    fn peek(&mut self, input: &str) -> Option<char> {
        if self.m_index >= input.len() {
            return None;
        }
        let c = input.chars().nth(self.m_index).unwrap(); // Accessing nth char
        Some(c)
    }
    
    fn check_buf(&mut self, buf : &Vec<char>) -> Option<Token> {
        let string_buf: String = buf.iter().collect();
        match string_buf.as_str() {
            "exit" => Some(Token::Exit {span : (0, self.m_index - 3)}),
            "(" => Some(Token::OpenParen),
            ")" => Some(Token::CloseParen),
            _ => None
        }
    }
}



pub enum Token {
    ID { name: String, span: (usize, usize) },
    Number { value: String, span: (usize, usize) },
    Exit {span: (usize, usize)},
    OpenParen,
    CloseParen
}

