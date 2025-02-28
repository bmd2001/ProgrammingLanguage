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



#[cfg(test)]
mod test_operator{
    use std::iter::zip;
    use super::*;
    
    fn all_operators(span: Span) -> Vec<Operator>{
        vec![
            Operator::Plus {span},
            Operator::Minus {span},
            Operator::Multiplication {span},
            Operator::Division {span},
            Operator::Exponent {span},
            Operator::Modulus {span},
            Operator::And {span},
            Operator::Or {span},
            Operator::Xor {span},
            Operator::Not {span},
            Operator::OpenBracket {span},
            Operator::ClosedBracket {span}
        ]
    }
    
    fn expected_precedence() -> Vec<usize> {
        vec![0, 0, 1, 1, 2, 1, 0, 0, 0, 3, 2, 2]
    }
    
    fn expected_format() -> Vec<&'static str>{
        vec!["+", "-", "*", "/", "^", "%", "&&", "||", "^|", "!!", "(", ")"]
    }
    
    #[test]
    fn test_precedence(){
        let dummy_span = Span::new(0, 0, 0);
        for (op, exp_precedence) in zip(all_operators(dummy_span), expected_precedence()){
            assert_eq!(op.precedence(), exp_precedence);
        }
    }
    
    #[test]
    fn test_associativity(){
        let dummy_span = Span::new(0, 0, 0);
        for op in all_operators(dummy_span){
            let exp_associativity = if matches!(op, Operator::Exponent {..} | Operator::Not {..}){
                "Right"
            } else { "Left"};
            assert_eq!(op.associativity(), exp_associativity);
        }
    }
    
    #[test]
    fn test_get_span(){
        let dummy_span = Span::new(0, 0, 0);
        for op in all_operators(dummy_span){
            assert_eq!(op.get_span(), dummy_span);
        }
    }
    
    #[test]
    fn test_formatting(){
        let dummy_span = Span::new(0, 0, 0);
        for (op, exp_format) in zip(all_operators(dummy_span), expected_format()){
            assert_eq!(format!("{}", op), exp_format);
        }
    }
}