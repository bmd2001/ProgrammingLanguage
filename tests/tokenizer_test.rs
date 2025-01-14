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
        Token::Exit { span: (0, (0, 3)) },
        Token::OpenParen { span: (0, 4) },
        Token::Number { value: "0".to_string(), span: (0, (5, 5)) },
        Token::CloseParen { span: (0, 6) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}
#[test]
fn test_multiple_whitespaces_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x       =     0  ");

    let expected_token = vec!(
        Token::ID { name: "x".to_string(), span: (0, (0, 0)) },
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::Equals {span : (0, 8)},
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::WhiteSpace,
        Token::Number { value: "0".to_string(), span: (0, (14, 14)) },
        Token::WhiteSpace,
        Token::WhiteSpace
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_variable_with_numbers(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x1=0");
    let expected_token = vec!(
        Token::ID { name: "x1".to_string(), span: (0, (0, 1)) },
        Token::Equals {span : (0, 2)},
        Token::Number { value: "0".to_string(), span: (0, (3, 3)) },
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_multiline_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x = 0\nexit(x)");

    let expected_token = vec!(
        Token::ID{ name: "x".to_string(), span: (0, (0, 0)) },
        Token::WhiteSpace,
        Token::Equals {span : (0, 2)},
        Token::WhiteSpace,
        Token::Number { value: "0".to_string(), span: (0, (4, 4)) },
        Token::NewLine,
        Token::Exit { span: (1, (0, 3)) },
        Token::OpenParen { span: (1, 4)},
        Token::ID { name: "x".to_string(), span: (1, (5, 5)) },
        Token::CloseParen { span: (1, 6) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_wrong_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("1x = 0");
    let expected_token = vec!(
        Token::Err {span: (0, (0, 1)) },
        Token::WhiteSpace,
        Token::Equals {span : (0, 3)},
        Token::WhiteSpace,
        Token::Number { value: "0".to_string(), span: (0, (5, 5)) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_operators(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("(+-*//**%)");

    let expected_token = vec!(
        Token::OpenParen {span: (0, 0)},
        Token::Operator(Operator::Plus {span: (0, 1)}),
        Token::Operator(Operator::Minus {span: (0, 2)}),
        Token::Operator(Operator::Multiplication {span: (0, 3)}),
        Token::Operator(Operator::Division {span: (0, (4, 5))}),
        Token::Operator(Operator::Exponent {span: (0, (6, 7))}),
        Token::Operator(Operator::Modulus {span: (0, 8)}),
        Token::CloseParen {span: (0, 9)}
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);

    tokenizer.tokenize("*** * ** ** *");
    let expected_token = vec!(
        Token::Operator(Operator::Exponent {span: (0, (0, 1))}),
        Token::Operator(Operator::Multiplication {span: (0, 2)}),
        Token::WhiteSpace,
        Token::Operator(Operator::Multiplication {span: (0, 4)}),
        Token::WhiteSpace,
        Token::Operator(Operator::Exponent {span: (0, (6, 7))}),
        Token::WhiteSpace,
        Token::Operator(Operator::Exponent {span: (0, (9, 10))}),
        Token::WhiteSpace,
        Token::Operator(Operator::Multiplication {span: (0, 12)}),
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

