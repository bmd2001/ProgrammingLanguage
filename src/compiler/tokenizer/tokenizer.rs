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
        let normalized_input = input.replace("\r", "");
        let mut buf = String::new();
        let mut chars = normalized_input.chars().peekable();
        while let Some(ch) = chars.next(){
            buf.push(ch);
            if let Some(token) = self.check_buf(&buf, &mut chars) {
                self.emit_token(token);
                buf.clear();
                continue;
            }
            if buf.chars().count() > 1 && ch == '\n'{
                let error_span = self.get_span(buf.chars().count()-1);
                self.m_row += 1;
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
            let error_span = self.get_span(buf.chars().count());
            self.emit_token(Token::Err {span: error_span });
        }
    }
    
    fn match_ch(&mut self, ch: char, peek: Option<&char>) -> Option<Token> {
        if ch == '\r' {
            return None;
        }

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
            "print" => {
                self.m_parenthesis_handler.activate_function_detector();
                Some(Token::Print {span : self.get_span(buf.len())})
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
            return if first.is_alphabetic()
                && buf.chars().all(char::is_alphanumeric)
                && !next_char.is_alphanumeric()
            {
                let value = buf.to_string();
                let value_len = buf.len();
                Some(Token::ID {
                    name: value,
                    span: self.get_span(value_len),
                })
            } else { None }
        }
        None
    }
    
    fn get_error(&mut self, buf: &str, input: &mut Peekable<Chars>) -> Option<Token>{
        let next_char = input.peek().unwrap_or(&' ');
        let mut chars = buf.chars();
        if let Some(first) = chars.next() {
            // If the buffer starts with a numeric but then contains only alphanumerics,
            // it may be an error if it doesn't form a proper number.
            return if first.is_numeric()
                && buf.chars().all(char::is_alphanumeric)
                && !next_char.is_alphanumeric()
            {
                let value_len = buf.len();
                Some(Token::Err {
                    span: self.get_span(value_len),
                })
            } else { None }
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



#[cfg(test)]
mod test_tokenizer{
    use super::*;
    
    mod test_functions{
        use super::*;
        
        #[test]
        fn test_get_span() {
            let mut temp_tokenizer = Tokenizer::new();
            let exp_span = Span::new(0, 0, 0);
            let res_span = temp_tokenizer.get_span(1);
            assert_eq!(exp_span, res_span);
            assert_eq!(temp_tokenizer.m_row, 0);

            let exp_span = Span::new(0, 0, 1);
            let res_span = temp_tokenizer.get_span(2);
            assert_eq!(exp_span, res_span);
            assert_eq!(temp_tokenizer.m_row, 1)
        }
        
        #[test]
        fn test_clear(){
            let mut temp_tokenizer = Tokenizer::new();
            let temp_span = Span::new(0, 0, 0);
            temp_tokenizer.m_tokens = vec![Token::NewLine {span: temp_span}];
            temp_tokenizer.m_line = 42;
            temp_tokenizer.m_row = 42;
            
            temp_tokenizer.clear();
            assert!(temp_tokenizer.m_tokens.is_empty());
            assert_eq!(temp_tokenizer.m_line, 0);
            assert_eq!(temp_tokenizer.m_row, 0);
        }
        
        #[test]
        fn test_match_ch(){
            let mut temp_tokenizer = Tokenizer::new();
            assert_eq!(temp_tokenizer.match_ch(' ', None), Some(Token::WhiteSpace {span: Span::new(0, 0, 0)}));
        }
        
        #[test]
        fn test_get_id(){
            let mut temp_tokenizer = Tokenizer::new();
            assert!(temp_tokenizer.get_id("", &mut "".chars().peekable()).is_none());
        }
        
        #[test]
        fn test_get_error(){
            let mut temp_tokenizer = Tokenizer::new();
            assert!(temp_tokenizer.get_error("", &mut "".chars().peekable()).is_none());
        }
    }
    
    
    mod test_tokenization{
        use super::*;
        

        #[test]
        fn test_empty_input() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("");
            assert!(tokenizer.get_tokens().is_empty());
        }

        #[test]
        fn test_exit_input() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("exit(0)");

            let expected_token = vec!(
                Token::Exit { span: Span::new(0, 0, 3) },
                Token::OpenBracket { span: Span::new(0, 4, 4) },
                Token::Number { value: "0".to_string(), span: Span::new(0, 5, 5) },
                Token::ClosedBracket { span: Span::new(0, 6, 6) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }
        
        #[test]
        fn test_print(){
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("print(0)");
            
            let expected_token = vec!(
                Token::Print { span: Span::new(0, 0, 4) },
                Token::OpenBracket { span: Span::new(0, 5, 5) },
                Token::Number { value: "0".to_string(), span: Span::new(0, 6, 6) },
                Token::ClosedBracket { span: Span::new(0, 7, 7) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }
        
        #[test]
        fn test_multiple_whitespaces_input() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("x       =     0  ");

            let expected_token = vec!(
                Token::ID { name: "x".to_string(), span: Span::new(0, 0, 0) },
                Token::WhiteSpace { span: Span::new(0, 1, 7) },
                Token::Equals {span: Span::new(0, 8, 8)},
                Token::WhiteSpace { span: Span::new(0, 9, 13) },
                Token::Number { value: "0".to_string(), span: Span::new(0, 14, 14) },
                Token::WhiteSpace { span: Span::new(0, 15, 16) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }

        #[test]
        fn test_variable_with_numbers(){
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("x1=0");
            let expected_token = vec!(
                Token::ID { name: "x1".to_string(), span: Span::new(0, 0, 1) },
                Token::Equals {span : Span::new(0, 2, 2)},
                Token::Number { value: "0".to_string(), span: Span::new(0, 3, 3) },
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }

        #[test]
        fn test_multiline_input() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("x = 0\nexit(x)\n{}\nprint(1)");

            let expected_token = vec!(
                Token::ID{ name: "x".to_string(), span: Span::new(0, 0, 0) },
                Token::WhiteSpace { span: Span::new(0, 1, 1)},
                Token::Equals {span : Span::new(0, 2, 2)},
                Token::WhiteSpace { span: Span::new(0, 3, 3)},
                Token::Number { value: "0".to_string(), span: Span::new(0, 4, 4) },
                Token::NewLine { span: Span::new(0, 5, 5)},
                Token::Exit { span: Span::new(1, 0, 3) },
                Token::OpenBracket { span: Span::new(1, 4, 4)},
                Token::ID { name: "x".to_string(), span: Span::new(1, 5, 5) },
                Token::ClosedBracket { span: Span::new(1, 6, 6) },
                Token::NewLine { span: Span::new(1, 7, 7)},
                Token::OpenCurlyBracket { span: Span::new(2, 0, 0)},
                Token::ClosedCurlyBracket { span: Span::new(2, 1, 1)},
                Token::NewLine { span: Span::new(2, 2, 2)},
                Token::Print { span: Span::new(3, 0, 4) },
                Token::OpenBracket { span: Span::new(3, 5, 5) },
                Token::Number { value: "1".to_string(), span: Span::new(3, 6, 6) },
                Token::ClosedBracket { span: Span::new(3, 7, 7) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }

        #[test]
        fn test_wrong_input() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("1x = 0");
            let expected_token = vec!(
                Token::Err {span: Span::new(0, 0, 1) },
                Token::WhiteSpace { span: Span::new(0, 2, 2)},
                Token::Equals {span : Span::new(0, 3, 3)},
                Token::WhiteSpace { span: Span::new(0, 4, 4)},
                Token::Number { value: "0".to_string(), span: Span::new(0, 5, 5) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }
        
        #[test]
        fn test_undefined_char(){
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("~∞");
            let expected_token = vec!(
                Token::Err {span: Span::new(0, 0, 1) }
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
            
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("~∞\n");
            let expected_token = vec!(
                Token::Err {span: Span::new(0, 0, 1) },
                Token::NewLine {span: Span::new(0, 2, 2)}
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }

        #[test]
        fn test_operators(){
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("(+-*//**%)");

            let expected_token = vec!(
                Token::Operator(Operator::OpenBracket {span: Span::new(0, 0, 0)}),
                Token::Operator(Operator::Plus {span: Span::new(0, 1, 1)}),
                Token::Operator(Operator::Minus {span: Span::new(0, 2, 2)}),
                Token::Operator(Operator::Multiplication {span: Span::new(0, 3, 3)}),
                Token::Operator(Operator::Division {span: Span::new(0, 4, 5)}),
                Token::Operator(Operator::Exponent {span: Span::new(0, 6, 7)}),
                Token::Operator(Operator::Modulus {span: Span::new(0, 8, 8)}),
                Token::Operator(Operator::ClosedBracket {span: Span::new(0, 9, 9)})
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);

            tokenizer.tokenize("*** * ** ** *");
            let expected_token = vec!(
                Token::Operator(Operator::Exponent {span: Span::new(0, 0, 1)}),
                Token::Operator(Operator::Multiplication {span: Span::new(0, 2, 2)}),
                Token::WhiteSpace { span: Span::new(0, 3, 3) },
                Token::Operator(Operator::Multiplication {span: Span::new(0, 4, 4)}),
                Token::WhiteSpace { span: Span::new(0, 5, 5) },
                Token::Operator(Operator::Exponent {span: Span::new(0, 6, 7)}),
                Token::WhiteSpace { span: Span::new(0, 8, 8) },
                Token::Operator(Operator::Exponent {span: Span::new(0, 9, 10)}),
                Token::WhiteSpace { span: Span::new(0, 11, 11) },
                Token::Operator(Operator::Multiplication {span: Span::new(0, 12, 12)}),
            );
            assert_eq!(tokenizer.get_tokens(), expected_token);
        }

        #[test]
        fn test_logical_operators() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("&&||!!^|");

            let expected_tokens = vec![
                Token::Operator(Operator::And { span: Span::new(0, 0, 1) }),
                Token::Operator(Operator::Or  { span: Span::new(0, 2, 3) }),
                Token::Operator(Operator::Not { span: Span::new(0, 4, 5) }),
                Token::Operator(Operator::Xor { span: Span::new(0, 6, 7) }),
            ];
            assert_eq!(tokenizer.get_tokens(), expected_tokens);
        }

        #[test]
        fn test_logical_operators_with_spacing() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("x && y || !! z ^| w");

            let expected_tokens = vec![
                Token::ID { name: "x".to_string(), span: Span::new(0, 0, 0) },
                Token::WhiteSpace { span: Span::new(0, 1, 1) },
                Token::Operator(Operator::And { span: Span::new(0, 2, 3) }),
                Token::WhiteSpace { span: Span::new(0, 4, 4) },
                Token::ID { name: "y".to_string(), span: Span::new(0, 5, 5) },
                Token::WhiteSpace { span: Span::new(0, 6, 6) },
                Token::Operator(Operator::Or { span: Span::new(0, 7, 8) }),
                Token::WhiteSpace { span: Span::new(0, 9, 9) },
                Token::Operator(Operator::Not { span: Span::new(0, 10, 11) }),
                Token::WhiteSpace { span: Span::new(0, 12, 12) },
                Token::ID { name: "z".to_string(), span: Span::new(0, 13, 13) },
                Token::WhiteSpace { span: Span::new(0, 14, 14) },
                Token::Operator(Operator::Xor { span: Span::new(0, 15, 16) }),
                Token::WhiteSpace { span: Span::new(0, 17, 17) },
                Token::ID { name: "w".to_string(), span: Span::new(0, 18, 18) },
            ];
            assert_eq!(tokenizer.get_tokens(), expected_tokens);
        }

        #[test]
        fn test_boolean_tokens() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("true false");

            let expected_tokens = vec![
                Token::Boolean { value: true, span: Span::new(0, 0, 3) },
                Token::WhiteSpace { span: Span::new(0, 4, 4) },
                Token::Boolean { value: false, span: Span::new(0, 5, 9) },
            ];
            assert_eq!(tokenizer.get_tokens(), expected_tokens);
        }

        #[test]
        fn test_logical_operators_multiline() {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize("x && y\n|| !! z\n^| w");

            let expected_tokens = vec![
                Token::ID { name: "x".to_string(), span: Span::new(0, 0, 0) },
                Token::WhiteSpace { span: Span::new(0, 1, 1) },
                Token::Operator(Operator::And { span: Span::new(0, 2, 3) }),
                Token::WhiteSpace { span: Span::new(0, 4, 4) },
                Token::ID { name: "y".to_string(), span: Span::new(0, 5, 5) },
                Token::NewLine { span: Span::new(0, 6, 6) },
                Token::Operator(Operator::Or { span: Span::new(1, 0, 1) }),
                Token::WhiteSpace { span: Span::new(1, 2, 2) },
                Token::Operator(Operator::Not { span: Span::new(1, 3, 4) }),
                Token::WhiteSpace { span: Span::new(1, 5, 5) },
                Token::ID { name: "z".to_string(), span: Span::new(1, 6, 6) },
                Token::NewLine { span: Span::new(1, 7, 7) },
                Token::Operator(Operator::Xor { span: Span::new(2, 0, 1) }),
                Token::WhiteSpace { span: Span::new(2, 2, 2) },
                Token::ID { name: "w".to_string(), span: Span::new(2, 3, 3) },
            ];

            assert_eq!(tokenizer.get_tokens(), expected_tokens);
        }
    }
}