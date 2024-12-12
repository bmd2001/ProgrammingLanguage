use BRS::compiler::tokenizer::{Tokenizer, Token, Operator};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// A global mutable Tokenizer wrapped in a Mutex
static TOKENIZER: Lazy<Mutex<Tokenizer>> = Lazy::new(|| {
    Mutex::new(Tokenizer::new()) // Adjust initialization as needed
});

#[test]
fn test_empty_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("");
    assert!(tokenizer.get_tokens().is_empty());
}

#[test]
fn test_exit_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("exit(0)");
    
    let expected_token = vec!(
        Token::Exit { span: (0, 0) },
        Token::OpenParen,
        Token::Number { value: "0".to_string(), span: (0, 5) },
        Token::CloseParen
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}
#[test]
fn test_multiple_whitespaces_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x       =     0  ");

    let expected_token = vec!(
        Token::ID { name: "x".to_string(), span: (0, 0) },
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::Equals,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::Number { value: "0".to_string(), span: (0, 14) },
        Token::WhiteSpace,
        Token::WhiteSpace
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_multiline_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x = 0\nexit(x)");

    let expected_token = vec!(
        Token::ID{ name: "x".to_string(), span: (0, 0) },
        Token::WhiteSpace,
        Token::Equals,
        Token::WhiteSpace,
        Token::Number { value: "0".to_string(), span: (0, 4) },
        Token::NewLine,
        Token::Exit { span: (1, 0) },
        Token::OpenParen,
        Token::ID { name: "x".to_string(), span: (1, 5) },
        Token::CloseParen
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_wrong_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("1x = 0");
    assert!(tokenizer.get_tokens().is_empty());
}

#[test]
fn test_operators(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x=1+2\ny=x-1\nz=3*y\nw=x//y\nexit(w)");

    let expected_token = vec!(
        Token::ID{ name: "x".to_string(), span: (0, 0) },
        Token::Equals,
        Token::Number { value: "1".to_string(), span: (0, 2) },
        Token::Operator(Operator::Plus),
        Token::Number { value: "2".to_string(), span: (0, 4) },
        Token::NewLine,
        Token::ID{ name: "y".to_string(), span: (1, 0) },
        Token::Equals,
        Token::ID{ name: "x".to_string(), span: (1, 2) },
        Token::Operator(Operator::Minus),
        Token::Number { value: "1".to_string(), span: (1, 4) },
        Token::NewLine,
        Token::ID{ name: "z".to_string(), span: (2, 0) },
        Token::Equals,
        Token::Number { value: "3".to_string(), span: (2, 2) },
        Token::Operator(Operator::Multiplication),
        Token::ID { name: "y".to_string(), span: (2, 4) },
        Token::NewLine,
        Token::ID{ name: "w".to_string(), span: (3, 0) },
        Token::Equals,
        Token::ID { name: "x".to_string(), span: (3, 2) },
        Token::Operator(Operator::Division),
        Token::ID { name: "y".to_string(), span: (3, 5) },
        Token::NewLine,
        Token::Exit { span: (4, 0) },
        Token::OpenParen,
        Token::ID { name: "w".to_string(), span: (4, 5) },
        Token::CloseParen
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

