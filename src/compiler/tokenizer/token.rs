use std::fmt;
use crate::compiler::span::Span;
use super::operator::Operator;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ID { name: String, span: Span },
    Number { value: String, span: Span },
    Boolean { value: bool, span: Span },
    Exit {span: Span},
    OpenBracket {span: Span},
    ClosedBracket {span: Span},
    OpenCurlyBracket {span: Span},
    ClosedCurlyBracket {span: Span},
    Equals {span: Span},
    Operator(Operator),
    WhiteSpace {span: Span},
    NewLine {span: Span},
    Err {span: Span}
}

impl Token {
    pub fn get_span(&self) -> Span {
        match self {
            Token::ID { span, .. }
            | Token::Number { span, .. }
            | Token::Boolean { span, .. }
            | Token::Exit { span }
            | Token::OpenBracket { span }
            | Token::ClosedBracket { span }
            | Token::OpenCurlyBracket { span }
            | Token::ClosedCurlyBracket { span }
            | Token::Equals { span }
            | Token::WhiteSpace { span }
            | Token::NewLine { span }
            | Token::Err { span } => span.clone(),
            Token::Operator(op) => op.get_span(),
        }
    }
}

// Implement Display for Token
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::ID { name, span } => write!(f, "ID({}, {:?})", name, span),
            Token::Number { value, span } => write!(f, "Number({}, {:?})", value, span),
            Token::Exit { .. } => write!(f, "exit()"),
            Token::OpenBracket { .. } => write!(f, "("),
            Token::ClosedBracket { .. } => write!(f, ")"),
            Token::OpenCurlyBracket { .. } => write!(f, "{{"),
            Token::ClosedCurlyBracket { .. } => write!(f, "}}"),
            Token::Equals {..} => write!(f, "="),
            Token::Operator(op) => write!(f, "{}", op),
            Token::WhiteSpace {..} => write!(f, " "),
            Token::NewLine {..} => write!(f, "\n"),
            _ => {write!(f, "err")}
        }
    }
}