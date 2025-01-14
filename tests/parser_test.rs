use BRS::compiler::parser::{NodeProgram, NodeStmt, NodeVariableAssignment, NodeArithmeticExpr, NodeBaseExpr, Parser};
use BRS::compiler::tokenizer::{Token, Tokenizer};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// A global mutable Tokenizer wrapped in a Mutex
static TOKENIZER: Lazy<Mutex<Tokenizer>> = Lazy::new(|| {
    Mutex::new(Tokenizer::new()) // Adjust initialization as needed
});

static NOT_VALID_EXIT_INPUTS: Lazy<Vec<&str>> = Lazy::new(|| vec![
    "exit(0",
    "exit0)",
    "exit (0)",
    "exita(0)",
    "exit(0\n)"
]);

static NOT_VALID_VAR_INPUTS: Lazy<Vec<&str>> = Lazy::new(|| vec![
    "1x=0",
    "x=(",
    "x=)",
    "x=1.0",
    "x=\n1"
]);

fn utility_create_parser(stmts: &str) -> Parser{
    let mut tokenizer = TOKENIZER.lock().unwrap();
    tokenizer.tokenize(stmts);
    Parser::new(tokenizer.get_tokens(), "".to_string(), stmts.to_string())
}

#[test]
fn test_empty_input() {
    let mut parser = utility_create_parser("");
    let prog = parser.parse().expect("For an empty input, the parser should not fail");
    assert!(prog.get_stmts().is_empty());
}

#[test]
fn test_exit() {
    let mut parser = utility_create_parser("exit(0)");
    let prog = parser.parse().expect("For an exit call \"exit(0)\", the parser should not fail");
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_wrong_exit(){
    for bad_exit_input in &*NOT_VALID_EXIT_INPUTS {
        let mut parser = utility_create_parser(&bad_exit_input);
        let result = parser.parse();
        assert!(result.is_none(), "For input `{bad_exit_input}`, the parser should fail but succeeded with: {:?}", result);
    }
}

#[test]
fn test_variable_assignment_base(){
    let mut parser = utility_create_parser("x=0");
    let prog = parser.parse().expect("For input \"x=0\", the parser should not fail");
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 1);
    assert_eq!(stmts[0], NodeStmt::ID(
        NodeVariableAssignment{
            variable: Token::ID { name: "x".to_string(), span: (0, (0, 0)) }, 
            value: NodeArithmeticExpr::Base(
                NodeBaseExpr::Num(
                    Token::Number {value: "0".to_string(),span: (0, (2, 2))}
                )
            )
        }
    ));
}

#[test]
fn test_variable_assignment_operation(){
    let mut parser = utility_create_parser("x=((3+5)*2 + (12//4))%7+(18//(6-3))*(2**3-4) + 10");
    let prog = parser.parse().expect("For input \"x=((3+5)*2 + (12//4))%7+(18//(6-3))*(2**3-4) + 10\", the parser should not fail");
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_wrong_variable_assignment(){
    for bad_var_input in &*NOT_VALID_VAR_INPUTS  {
        let mut parser = utility_create_parser(&bad_var_input);
        let result = parser.parse();
        assert!(result.is_none(), "For input `{bad_var_input}`, the parser should fail but succeeded with: {:?}", result);
    }
}