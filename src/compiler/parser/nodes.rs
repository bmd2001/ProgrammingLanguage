use std::fmt;
use std::fmt::{Formatter};
use either::{Either, Left};
use crate::compiler::Token;
use crate::compiler::tokenizer::Operator;

#[derive(Debug, PartialEq)]
pub struct NodeProgram{
    pub(crate) stmts: Vec<NodeStmt>
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeStmt {
    Exit(NodeExit),
    Print(NodePrint),
    ID(NodeVariableAssignment),
    Scope(NodeScope)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeExit {
    pub(crate) expr: NodeArithmeticExpr
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodePrint {
    pub(crate) expr: NodeArithmeticExpr
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeVariableAssignment {
    pub variable: Token,
    pub value: NodeArithmeticExpr
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeScope {
    pub stmts: Vec<NodeStmt>
}
#[derive(Clone, Debug, PartialEq)]
pub enum NodeArithmeticExpr {
    Base(NodeBaseExpr),
    Operation(NodeArithmeticOperation)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeArithmeticOperation {
    pub(crate) lhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>,
    pub(crate) rhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>,
    pub(crate) op: Operator,
    pub(crate) result_type: ResultType
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResultType{
    Numeric,
    Boolean
}

impl ResultType {
    pub fn as_str(&self) -> &str{
        match self {
            ResultType::Numeric => {"num"}
            ResultType::Boolean => {"bool"}
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum NodeBaseExpr {
    Num(Token),
    ID(Token),
    Bool(Token),
}

impl NodeProgram{
    pub fn get_stmts(& self) -> Vec<NodeStmt>{
        self.stmts.clone()
    }
}

impl fmt::Display for NodeVariableAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Token::ID { name, .. } = &self.variable {
            write!(f, "{} = {}", name, self.value)
        } else {
            write!(f, "Invalid variable token")
        }
    }
}

impl fmt::Display for NodeArithmeticExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeArithmeticExpr::Base(base) => {write!(f, "{base}")}
            NodeArithmeticExpr::Operation(op) => {write!(f, "{op}")}
        }
    }
}

impl fmt::Display for NodeArithmeticOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the operation as `lhs op rhs`
        let lhs_str = if matches!(self.lhs, Left(..)) {
            format!("({})", self.lhs)
        } else{
            format!("{}", self.lhs)
        };
        let rhs_str = if matches!(self.rhs, Left(..)) {
            format!("({})", self.rhs)
        } else{
            format!("{}", self.rhs)
        };
        write!(
            f,
            "{} {} {}",
            lhs_str,
            self.op,
            rhs_str
        )
    }
}

impl fmt::Display for NodeBaseExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeBaseExpr::Num(Token::Number { value, .. }) => write!(f, "{}", value),
            NodeBaseExpr::ID(Token::ID { name, .. }) => write!(f, "{}", name),
            NodeBaseExpr::Bool(Token::Boolean { value, .. }) => write!(f, "{}", value),
            _ => write!(f, "Invalid base expression"),
        }
    }
}

impl fmt::Display for NodeExit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "exit({})", self.expr)
    }
}

impl fmt::Display for NodePrint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "print({})", self.expr)
    }
}

impl fmt::Display for NodeStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeStmt::Exit(exit) => write!(f, "{}", exit),
            NodeStmt::Print(print) => write!(f, "{}", print),
            NodeStmt::ID(var_assign) => write!(f, "{}", var_assign),
            NodeStmt::Scope(scope) => {
                write!(f, "{{")?;
                for stmt in &scope.stmts {
                    write!(f, "{}", stmt)?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl fmt::Display for NodeProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let stmt_count = self.stmts.len();
        for (i, stmt) in self.stmts.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < stmt_count - 1 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}



#[cfg(test)]
mod test_nodes {
    use crate::compiler::span::Span;
    use super::*;
    
    #[test]
    fn test_formatting_result_type(){
        assert_eq!(ResultType::Numeric.as_str(), "num");
        assert_eq!(ResultType::Boolean.as_str(), "bool")
    }

    #[test]
    fn test_formatting_node_arithmetic_expr_base() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token = Token::Number { value: "5".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::Num(num_token.clone());

        // Testing NodeBaseExpr::Num
        let formatted = format!("{}", base_expr);
        assert_eq!(formatted, "5");

        let id_token = Token::ID { name: "x".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::ID(id_token);

        // Testing NodeBaseExpr::ID
        let formatted = format!("{}", base_expr);
        assert_eq!(formatted, "x");

        let bool_token = Token::Boolean { value: true, span: dummy_span };
        let base_expr = NodeBaseExpr::Bool(bool_token);

        // Testing NodeBaseExpr::Bool
        let formatted = format!("{}", base_expr);
        assert_eq!(formatted, "true");
    }

    #[test]
    fn test_formatting_node_arithmetic_expr_operation() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token_1 = Token::Number { value: "5".to_string(), span: dummy_span };
        let num_token_2 = Token::Number { value: "3".to_string(), span: dummy_span };

        let base_expr_1 = NodeBaseExpr::Num(num_token_1);
        let base_expr_2 = NodeBaseExpr::Num(num_token_2);

        let operation = NodeArithmeticOperation {
            lhs: Either::Right(base_expr_1.clone()),
            rhs: Either::Right(base_expr_2.clone()),
            op: Operator::Plus { span: dummy_span }, // Assuming Operator::Add is an addition operator
            result_type: ResultType::Numeric,
        };

        let formatted = format!("{}", operation);
        assert_eq!(formatted, "5 + 3");
    }

    #[test]
    fn test_formatting_node_exit() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token = Token::Number { value: "10".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::Num(num_token);

        let exit_node = NodeExit { expr: NodeArithmeticExpr::Base(base_expr) };

        // Testing NodeExit
        let formatted = format!("{}", NodeStmt::Exit(exit_node));
        assert_eq!(formatted, "exit(10)");
    }

    #[test]
    fn test_formatting_node_variable_assignment() {
        let dummy_span = Span::new(0, 0, 0);
        let var_token = Token::ID { name: "x".to_string(), span: dummy_span };
        let num_token = Token::Number { value: "1".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::Num(num_token);
        let var_assign = NodeVariableAssignment { variable: var_token, value: NodeArithmeticExpr::Base(base_expr.clone()) };

        // Testing valid variable assignment
        assert_eq!(format!("{}", var_assign), "x = 1");

        // Testing invalid variable token
        let wrong_var_token = Token::NewLine { span: dummy_span };
        let invalid_var_assign = NodeVariableAssignment { variable: wrong_var_token, value: NodeArithmeticExpr::Base(base_expr) };
        assert_eq!(format!("{}", invalid_var_assign), "Invalid variable token");
    }

    #[test]
    fn test_formatting_node_stmt_scope() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token = Token::Number { value: "1".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::Num(num_token);

        let var_token = Token::ID { name: "x".to_string(), span: dummy_span };
        let var_assign = NodeVariableAssignment { variable: var_token, value: NodeArithmeticExpr::Base(base_expr) };
        let scope_stmt = NodeStmt::Scope(NodeScope { stmts: vec![NodeStmt::ID(var_assign)] });

        // Testing Scope Statement
        let formatted = format!("{}", scope_stmt);
        assert_eq!(formatted, "{x = 1}");
    }
    
    #[test]
    fn test_bad_node_base_expr(){
        let dummy_span = Span::new(0, 0, 0);
        let bad_base_expr = NodeBaseExpr::Num(Token::Err {span: dummy_span});
        
        assert_eq!(format!("{}", bad_base_expr), "Invalid base expression")
    }

    #[test]
    fn test_formatting_node_arithmetic_expr_operation_with_nested_expr() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token_1 = Token::Number { value: "5".to_string(), span: dummy_span };
        let num_token_2 = Token::Number { value: "3".to_string(), span: dummy_span };

        let base_expr_1 = NodeBaseExpr::Num(num_token_1);
        let base_expr_2 = NodeBaseExpr::Num(num_token_2);
        let operation_1 = NodeArithmeticOperation {
            lhs: Either::Right(base_expr_1.clone()),
            rhs: Either::Right(base_expr_2.clone()),
            op: Operator::Plus {span: dummy_span},
            result_type: ResultType::Numeric,
        };

        let operation_2 = NodeArithmeticOperation {
            lhs: Either::Left(Box::new(operation_1.clone())),
            rhs: Either::Left(Box::new(operation_1.clone())),
            op: Operator::Multiplication {span: dummy_span},
            result_type: ResultType::Numeric,
        };

        // Testing nested arithmetic operation
        let formatted = format!("{}", NodeArithmeticExpr::Operation(operation_2));
        assert_eq!(formatted, "(5 + 3) * (5 + 3)");
    }

    #[test]
    fn test_formatting_node_program() {
        let dummy_span = Span::new(0, 0, 0);
        let num_token = Token::Number { value: "5".to_string(), span: dummy_span };
        let base_expr = NodeBaseExpr::Num(num_token);

        let var_token = Token::ID { name: "x".to_string(), span: dummy_span };
        let var_assign = NodeVariableAssignment { variable: var_token, value: NodeArithmeticExpr::Base(base_expr) };
        let stmt = NodeStmt::ID(var_assign);

        let program = NodeProgram { stmts: vec![stmt.clone(), stmt] };

        // Testing NodeProgram formatting
        let formatted = format!("{}", program);
        assert_eq!(formatted, "x = 5\nx = 5");
    }
}
