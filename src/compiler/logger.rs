use std::collections::HashMap;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

pub trait Logger {
    fn new(file_name: String, code: String) -> Self;
    fn log_error(&self, message: &str, span: (usize, (usize, usize)));
}


pub struct ParserLogger{
    file_name: String,
    source: Source
}

impl ParserLogger {
    pub fn log_errors(&self, errors: Vec<(ParserErrorType, (usize, (usize, usize)))>){
        for (error, span) in errors {
            self.log_error(error.message(), span)
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ParserErrorType{
    ErrInvalidStatement,
    ErrExitOpenParenthesisMissing,
    ErrExitClosedParenthesisMissing,
    ErrUnexpectedToken,
    ErrExpressionOpenParenthesisMissing,
    ErrExpressionClosedParenthesisMissing,
    ErrMissingOperand,
}

impl ParserErrorType {
    pub fn message(&self) -> &'static str {
        match self {
            ParserErrorType::ErrInvalidStatement => "Invalid statement",
            ParserErrorType::ErrExitOpenParenthesisMissing => "Exit '(' is missing.",
            ParserErrorType::ErrExitClosedParenthesisMissing => "Exit ')' is missing.",
            ParserErrorType::ErrUnexpectedToken => "Unexpected character sequence found here.",
            ParserErrorType::ErrExpressionOpenParenthesisMissing => "Mismatched Parenthesis: ( is missing",
            ParserErrorType::ErrExpressionClosedParenthesisMissing => "Mismatched Parenthesis: ) is missing",
            ParserErrorType::ErrMissingOperand => "Missing operand for operator.",
        }
    }
}

impl Logger for ParserLogger{
    fn new(file_name: String, code: String) -> ParserLogger{
        ParserLogger{ file_name, source: Source::from(code)}
    }
    
    fn log_error(&self, message: &str, span: (usize, (usize, usize))) {
        let (line_i, (row_start, row_end)) = span;
        let offset = self.source.line(line_i).expect("Custom Span logic returned wrong line ID").offset();
        Report::build(ReportKind::Error, (self.file_name.as_str(), offset+row_start..offset+row_end))
            .with_message(message)
            .with_label(
                Label::new((self.file_name.as_str(), offset+row_start..offset+row_end))
                    .with_message(message)
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((self.file_name.as_str(), self.source.clone()))
            .unwrap();
    }
}