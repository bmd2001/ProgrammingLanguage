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
        Token::OpenParen { span: (0, (4, 4)) },
        Token::Number { value: "0".to_string(), span: (0, (5, 5)) },
        Token::CloseParen { span: (0, (6, 6)) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}
#[test]
fn test_multiple_whitespaces_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x       =     0  ");

    let expected_token = vec!(
        Token::ID { name: "x".to_string(), span: (0, (0, 0)) },
        Token::WhiteSpace { span: (0, (1, 1)) },
        Token::WhiteSpace { span: (0, (2, 2)) },
        Token::WhiteSpace { span: (0, (3, 3)) },
        Token::WhiteSpace { span: (0, (4, 4)) },
        Token::WhiteSpace { span: (0, (5, 5)) },
        Token::WhiteSpace { span: (0, (6, 6)) },
        Token::WhiteSpace { span: (0, (7, 7)) },
        Token::Equals {span : (0, (8, 8))},
        Token::WhiteSpace { span: (0, (9, 9)) },
        Token::WhiteSpace { span: (0, (10, 10)) },
        Token::WhiteSpace { span: (0, (11, 11)) },
        Token::WhiteSpace { span: (0, (12, 12)) },
        Token::WhiteSpace { span: (0, (13, 13)) },
        Token::Number { value: "0".to_string(), span: (0, (14, 14)) },
        Token::WhiteSpace { span: (0, (15, 15)) },
        Token::WhiteSpace { span: (0, (16, 16)) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_variable_with_numbers(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x1=0");
    let expected_token = vec!(
        Token::ID { name: "x1".to_string(), span: (0, (0, 1)) },
        Token::Equals {span : (0, (2, 2))},
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
        Token::WhiteSpace { span: (0, (1, 1))},
        Token::Equals {span : (0, (2, 2))},
        Token::WhiteSpace { span: (0, (3, 3))},
        Token::Number { value: "0".to_string(), span: (0, (4, 4)) },
        Token::NewLine { span: (0, (5, 5))},
        Token::Exit { span: (1, (0, 3)) },
        Token::OpenParen { span: (1, (4, 4))},
        Token::ID { name: "x".to_string(), span: (1, (5, 5)) },
        Token::CloseParen { span: (1, (6, 6)) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_wrong_input() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("1x = 0");
    let expected_token = vec!(
        Token::Err {span: (0, (0, 1)) },
        Token::WhiteSpace { span: (0, (2, 2))},
        Token::Equals {span : (0, (3, 3))},
        Token::WhiteSpace { span: (0, (4, 4))},
        Token::Number { value: "0".to_string(), span: (0, (5, 5)) }
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_operators(){
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("(+-*//**%)");

    let expected_token = vec!(
        Token::OpenParen {span: (0, (0, 0))},
        Token::Operator(Operator::Plus {span: (0, (1, 1))}),
        Token::Operator(Operator::Minus {span: (0, (2, 2))}),
        Token::Operator(Operator::Multiplication {span: (0, (3, 3))}),
        Token::Operator(Operator::Division {span: (0, (4, 5))}),
        Token::Operator(Operator::Exponent {span: (0, (6, 7))}),
        Token::Operator(Operator::Modulus {span: (0, (8, 8))}),
        Token::CloseParen {span: (0, (9, 9))}
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);

    tokenizer.tokenize("*** * ** ** *");
    let expected_token = vec!(
        Token::Operator(Operator::Exponent {span: (0, (0, 1))}),
        Token::Operator(Operator::Multiplication {span: (0, (2, 2))}),
        Token::WhiteSpace { span: (0, (3, 3)) },
        Token::Operator(Operator::Multiplication {span: (0, (4, 4))}),
        Token::WhiteSpace { span: (0, (5, 5)) },
        Token::Operator(Operator::Exponent {span: (0, (6, 7))}),
        Token::WhiteSpace { span: (0, (8, 8)) },
        Token::Operator(Operator::Exponent {span: (0, (9, 10))}),
        Token::WhiteSpace { span: (0, (11, 11)) },
        Token::Operator(Operator::Multiplication {span: (0, (12, 12))}),
    );
    assert_eq!(tokenizer.get_tokens(), expected_token);
}

#[test]
fn test_logical_operators() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("&&||!!^|");
    
    let expected_tokens = vec![
        Token::Operator(Operator::And { span: (0, (0, 1)) }),
        Token::Operator(Operator::Or  { span: (0, (2, 3)) }),
        Token::Operator(Operator::Not { span: (0, (4, 5)) }),
        Token::Operator(Operator::Xor { span: (0, (6, 7)) }),
    ];
    assert_eq!(tokenizer.get_tokens(), expected_tokens);
}

#[test]
fn test_logical_operators_with_spacing() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x && y || !! z ^| w");
    
    let expected_tokens = vec![
        Token::ID { name: "x".to_string(), span: (0, (0, 0)) },
        Token::WhiteSpace { span: (0, (1, 1)) },
        Token::Operator(Operator::And { span: (0, (2, 3)) }),
        Token::WhiteSpace { span: (0, (4, 4)) },
        Token::ID { name: "y".to_string(), span: (0, (5, 5)) },
        Token::WhiteSpace { span: (0, (6, 6)) },
        Token::Operator(Operator::Or { span: (0, (7, 8)) }),
        Token::WhiteSpace { span: (0, (9, 9)) },
        Token::Operator(Operator::Not { span: (0, (10, 11)) }),
        Token::WhiteSpace { span: (0, (12, 12)) },
        Token::ID { name: "z".to_string(), span: (0, (13, 13)) },
        Token::WhiteSpace { span: (0, (14, 14)) },
        Token::Operator(Operator::Xor { span: (0, (15, 16)) }),
        Token::WhiteSpace { span: (0, (17, 17)) },
        Token::ID { name: "w".to_string(), span: (0, (18, 18)) },
    ];
    assert_eq!(tokenizer.get_tokens(), expected_tokens);
}

#[test]
fn test_logical_operators_multiline() {
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize("x && y\n|| !! z\n^| w");
    
    let expected_tokens = vec![
        Token::ID { name: "x".to_string(), span: (0, (0, 0)) },
        Token::WhiteSpace { span: (0, (1, 1)) },
        Token::Operator(Operator::And { span: (0, (2, 3)) }),
        Token::WhiteSpace { span: (0, (4, 4)) },
        Token::ID { name: "y".to_string(), span: (0, (5, 5)) },
        Token::NewLine { span: (0, (6, 6)) },
        Token::Operator(Operator::Or { span: (1, (0, 1)) }),
        Token::WhiteSpace { span: (1, (2, 2)) },
        Token::Operator(Operator::Not { span: (1, (3, 4)) }),
        Token::WhiteSpace { span: (1, (5, 5)) },
        Token::ID { name: "z".to_string(), span: (1, (6, 6)) },
        Token::NewLine { span: (1, (7, 7)) },
        Token::Operator(Operator::Xor { span: (2, (0, 1)) }),
        Token::WhiteSpace { span: (2, (2, 2)) },
        Token::ID { name: "w".to_string(), span: (2, (3, 3)) },
    ];
    
    assert_eq!(tokenizer.get_tokens(), expected_tokens);
}
