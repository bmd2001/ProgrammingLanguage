use BRS::compiler::parser::{NodeStmt, NodeVariableAssignment, NodeArithmeticExpr, NodeBaseExpr, Parser};
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

static NOT_VALID_SCOPE_INPUTS: Lazy<Vec<&str>> = Lazy::new(|| vec![
    "{x=0",
    "x=0}"
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

#[test]
fn test_missing_operand_logical() {
    let input = "x = a &&";
    let mut parser = utility_create_parser(input);
    let result = parser.parse();
    assert!(result.is_none(), "Parser should fail for missing operand in logical expression 'x = a &&'");
}

#[test]
fn test_missing_operand_unary() {
    let input = "x = !!";
    let mut parser = utility_create_parser(input);
    let result = parser.parse();
    assert!(result.is_none(), "Parser should fail for missing operand in unary expression 'x = !!'");
}

#[test]
fn test_boolean_expression_parsing() {
    let input = "x = true";
    let mut parser = utility_create_parser(input);
    let prog = parser.parse().expect("Parser should succeed for 'x = true'");
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 1, "Expected one statement");

    let ast_string = format!("{}", stmts[0]);
    assert_eq!(ast_string, "x = true", "AST should correctly represent the boolean expression");
}

#[test]
fn test_valid_boolean_logical_expression() {
    let input = "x = true && false";
    let mut parser = utility_create_parser(input);
    let prog = parser.parse().expect("Parser should succeed for a boolean logical expression");
    let ast_string = format!("{}", prog.get_stmts()[0]);
    assert_eq!(ast_string, "x = true && false", "AST should correctly represent the boolean logical expression");
}

#[test]
fn test_invalid_logical_expression_non_boolean_operand() {
    let input = "x = 1 && true";
    let mut parser = utility_create_parser(input);
    let result = parser.parse();
    assert!(result.is_none(), "Parser should fail when non-boolean operand is used with a logical operator");
}

#[test]
fn test_scope(){
    let input = "x = 1\n{x = 0\nexit(x)}\nexit(x)";
    let mut parser = utility_create_parser(input);
    let prog = parser.parse().expect(&format!("For input {}, the parser should not fail", input));
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 3, "Expected 3 statements, found {}", stmts.len());
    assert!(matches!(&stmts[1], NodeStmt::Scope(_)), "Expected a scope statement, found {:?}", stmts[0]);
    let scope = match &stmts[1] {
        NodeStmt::Scope(s) => s,
        _ => panic!("Expected a scope statement, found {:?}", stmts[0])
    };
    assert_eq!(scope.stmts.len(), 2, "Expected 2 statements in the scope, found {}", scope.stmts.len());
}

#[test]
fn test_scope_nested(){
    let input = "{x = 1\n{x = 0\nexit(x)}}";
    let mut parser = utility_create_parser(input);
    let prog = parser.parse().expect(&format!("For input {}, the parser should not fail", input));
    let stmts = prog.get_stmts();
    assert_eq!(stmts.len(), 1, "Expected 1 statements, found {}", stmts.len());
    assert!(matches!(&stmts[0], NodeStmt::Scope(_)), "Expected a scope statement, found {:?}", stmts[0]);
    let scope = match &stmts[0] {
        NodeStmt::Scope(s) => s,
        _ => panic!("Expected a scope statement, found {:?}", stmts[0])
    };
    let scope_stmts = &scope.stmts;
    assert_eq!(scope_stmts.len(), 2, "Expected 2 statements, found {}", scope_stmts.len());
    assert!(matches!(scope_stmts[0], NodeStmt::ID(_)), "Expected a variable assignment statement, found {:?}", scope_stmts[0]);
    assert!(matches!(scope_stmts[1], NodeStmt::Scope(_)), "Expected a scope statement, found {:?}", scope_stmts[1]);
    let inner_scope =  match &scope_stmts[1]{
        NodeStmt::Scope(s) => s,
        _ => panic!("Expected a scope statement, found {:?}", stmts[0])
    };
    let inner_scope_stmts = &inner_scope.stmts;
    assert_eq!(inner_scope_stmts.len(), 2, "Expected 2 statements, found {}", stmts.len());
    assert!(matches!(inner_scope_stmts[0], NodeStmt::ID(_)), "Expected a variable assignment statement, found {:?}", inner_scope_stmts[0]);
    assert!(matches!(inner_scope_stmts[1], NodeStmt::Exit(_)), "Expected a scope statement, found {:?}", inner_scope_stmts[1]);
}

#[test]
fn test_scope_invalid(){
    for bad_scope_input in &*NOT_VALID_SCOPE_INPUTS {
        let mut parser = utility_create_parser(bad_scope_input);
        let result = parser.parse();
        assert!(result.is_none(), "For input `{bad_scope_input}`, the parser should fail but succeeded with: {:?}", result);
    }
}