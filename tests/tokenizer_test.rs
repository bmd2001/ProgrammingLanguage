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
    assert_eq!(tokenizer.get_tokens(), vec!(Token::Err));
}

#[test]
fn test_operators(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("(+-*//**%)");

    let expected_token = vec!(
        Token::OpenParen,
        Token::Operator(Operator::Plus),
        Token::Operator(Operator::Minus),
        Token::Operator(Operator::Multiplication),
        Token::Operator(Operator::Division),
        Token::Operator(Operator::Exponent),
        Token::Operator(Operator::Modulus),
        Token::CloseParen
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);

    tokenizer.tokenize("*** * ** ** *");
    let expected_token = vec!(
        Token::Operator(Operator::Exponent),
        Token::Operator(Operator::Multiplication),
        Token::WhiteSpace,
        Token::Operator(Operator::Multiplication),
        Token::WhiteSpace,
        Token::Operator(Operator::Exponent),
        Token::WhiteSpace,
        Token::Operator(Operator::Exponent),
        Token::WhiteSpace,
        Token::Operator(Operator::Multiplication)
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

