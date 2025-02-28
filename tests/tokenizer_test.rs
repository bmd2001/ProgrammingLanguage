use BRS::compiler::tokenizer::{Tokenizer, Token, Operator};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use BRS::compiler::span::Span;

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
        Token::Exit { span: Span::new(0, 0, 3) },
        Token::OpenBracket { span: Span::new(0, 4, 4) },
        Token::Number { value: "0".to_string(), span: Span::new(0, 5, 5) },
        Token::ClosedBracket { span: Span::new(0, 6, 6) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}
#[test]
fn test_multiple_whitespaces_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x = 0\nexit(x)\n{}");

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
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_wrong_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
fn test_operators(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
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
    let mut tokenizer = TOKENIZER.lock().unwrap();
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