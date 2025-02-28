use super::operator::Operator;
use crate::compiler::span::Span;
use std::fmt;

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
            Token::Boolean {value, span} => write!(f, "Boolean({}, {:?})", value, span),
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



#[cfg(test)]
mod test_token{
    use std::iter::zip;
    use super::*;

    fn all_tokens(span: Span) -> Vec<Token> {
        vec![
            Token::ID { name: "x".to_string(), span },
            Token::Number { value: "42".to_string(), span },
            Token::Boolean { value: true, span },
            Token::Exit { span },
            Token::OpenBracket { span },
            Token::ClosedBracket { span },
            Token::OpenCurlyBracket { span },
            Token::ClosedCurlyBracket { span },
            Token::Equals { span },
            Token::Operator(Operator::Plus { span }),
            Token::WhiteSpace { span },
            Token::NewLine { span },
            Token::Err { span },
        ]
    }
    
    fn expected_format() -> Vec<&'static str>{
        vec![
            "ID(x, Span { m_line: 0, m_start: 0, m_end: 0 })",
            "Number(42, Span { m_line: 0, m_start: 0, m_end: 0 })",
            "Boolean(true, Span { m_line: 0, m_start: 0, m_end: 0 })",
            "exit()",
            "(",
            ")",
            "{",
            "}",
            "=",
            "+",
            " ",
            "\n",
            "err"
        ]
    }
    
    #[test]
    fn test_get_span(){
        let dummy_span = Span::new(0, 0, 0);
        for token in all_tokens(dummy_span){
            assert_eq!(token.get_span(), dummy_span)
        }
    }
    
    #[test]
    fn test_formatting(){
        let dummy_span = Span::new(0,0, 0);
        for (token, exp_format) in zip(all_tokens(dummy_span), expected_format()){
            assert_eq!(format!("{}", token), exp_format);
        }
    }
}