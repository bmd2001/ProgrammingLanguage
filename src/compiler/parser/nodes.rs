use std::fmt;
use either::{Either, Left};
use crate::compiler::Token;

#[derive(Debug, PartialEq)]
pub struct NodeProgram{
    pub(crate) stmts: Vec<NodeStmt>
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeStmt {
    Exit(NodeExit),
    ID(NodeVariableAssignment),
    Scope(NodeScope)
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeExit {
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
    pub(crate) op: Token
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

impl fmt::Display for NodeStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeStmt::Exit(exit) => write!(f, "exit({})", exit.expr),
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