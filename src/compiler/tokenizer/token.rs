use std::fmt;
use super::operator::Operator;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    ID { name: String, span: (usize, (usize, usize)) },
    Number { value: String, span: (usize, (usize, usize)) },
    Boolean { value: bool, span: (usize, (usize, usize)) },
    Exit {span: (usize, (usize, usize))},
    OpenBracket {span: (usize, (usize, usize))},
    ClosedBracket {span: (usize, (usize, usize))},
    OpenCurlyBracket {span: (usize, (usize, usize))},
    ClosedCurlyBracket {span: (usize, (usize, usize))},
    Equals {span: (usize, (usize, usize))},
    Operator(Operator),
    WhiteSpace {span: (usize, (usize, usize))},
    NewLine {span: (usize, (usize, usize))},
    Err {span: (usize, (usize, usize))}
}

impl Token {
    pub fn get_span(&self) -> (usize, (usize, usize)) {
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
            | Token::NewLine { span } => *span,
            Token::Err { span } => {
                let (line, (start, end)) = *span;
                (line, (start, end+1))
            },
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