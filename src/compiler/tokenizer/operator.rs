use crate::compiler::span::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Operator {
    Plus {span: Span},
    Minus {span: Span},
    Multiplication {span: Span},
    Division {span: Span},
    Exponent {span: Span},
    Modulus {span: Span},
    And {span: Span},
    Or {span: Span},
    Xor {span: Span},
    Not {span: Span},
    OpenBracket {span: Span},
    ClosedBracket {span: Span}
}

impl Operator {
    pub fn get_span(&self) -> Span {
        match self {
            Operator::Plus { span }
            | Operator::Minus { span }
            | Operator::Multiplication { span }
            | Operator::Division { span }
            | Operator::Exponent { span }
            | Operator::Modulus { span }
            | Operator::And { span }
            | Operator::Or { span }
            | Operator::Xor { span }
            | Operator::Not { span }
            | Operator::OpenBracket { span }
            | Operator::ClosedBracket { span } => *span,
        }
    }
}

// Implement Display for Operator
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self {
            Operator::Plus { span: _ } => "+",
            Operator::Minus { span: _ } => "-",
            Operator::Multiplication { span: _ } => "*",
            Operator::Division { span: _ } => "/",
            Operator::Exponent { span: _ } => "^",
            Operator::Modulus { span: _ } => "%",
            Operator::And { .. } => "&&",
            Operator::Or { .. } => "||",
            Operator::Xor { .. } => "^|",
            Operator::Not { .. } => "!!",
            Operator::OpenBracket { span: _ } => "(",
            Operator::ClosedBracket { span: _ } => ")"

        };
        write!(f, "{}", symbol)
    }
}

impl Operator {
    pub fn precedence(self) -> usize {
        match self {
            Operator::Plus { .. } | Operator::Minus { .. } | Operator::And { .. } | Operator::Or { .. } | Operator::Xor { .. } => {0}
            Operator::Multiplication { .. } | Operator::Division { .. } | Operator::Modulus { .. } => {1}
            Operator::OpenBracket { .. } | Operator::ClosedBracket { .. } | Operator::Exponent { .. } => {2}
            Operator::Not { .. } => 3
        }
    }

    pub fn associativity(self) -> String {
        match self{
            Operator::Exponent { .. } | Operator::Not { .. } => {"Right".to_string()}
            _ => {"Left".to_string()}
        }
    }
}