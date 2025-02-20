use std::sync::{Mutex, OnceLock};
use ariadne::{Color, Label, Report, ReportKind, Source};
use crate::compiler::logger::Logger;


pub struct ParserLogger{
    file_name: String,
    source: Source,
    errors: Vec<(ParserErrorType, (usize, (usize, usize)))>
}

impl ParserLogger {
    pub fn failed_parsing(&self) -> bool {
        !self.errors.is_empty()
    }
    
    fn log_errors(&self){
        // Check if the code is being run with a test profile
        let is_test_profile = std::thread::current().name().map_or(false, |name| name.starts_with("test"));
        if !is_test_profile {
            for (error, span) in self.errors.clone() {
                self.log_error(error.message(), span)
            }
        }
    }
    
    fn report_error(&mut self, error: (ParserErrorType, (usize, (usize, usize)))) {
        self.errors.push(error);
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ParserErrorType{
    ErrInvalidStatement,
    ErrExitOpenBracketMissing,
    ErrExitClosedBracketMissing,
    ErrUnexpectedToken,
    ErrExpressionOpenBracketMissing,
    ErrExpressionClosedBracketMissing,
    ErrScopeClosesCurlyBracketMissing,
    ErrMissingOperand,
    ErrTypeMismatch,
}

impl ParserErrorType {
    pub fn message(&self) -> &'static str {
        match self {
            ParserErrorType::ErrInvalidStatement => "Invalid statement",
            ParserErrorType::ErrExitOpenBracketMissing => "Exit '(' is missing.",
            ParserErrorType::ErrExitClosedBracketMissing => "Exit ')' is missing.",
            ParserErrorType::ErrUnexpectedToken => "Unexpected character sequence found here.",
            ParserErrorType::ErrExpressionOpenBracketMissing => "Mismatched Parenthesis: ( is missing",
            ParserErrorType::ErrExpressionClosedBracketMissing => "Mismatched Parenthesis: ) is missing",
            ParserErrorType::ErrScopeClosesCurlyBracketMissing => "Scope is initialized but never closes",
            ParserErrorType::ErrMissingOperand => "Missing operand for operator.",
            ParserErrorType::ErrTypeMismatch => "Type mismatch in expression.",
        }
    }
}

impl Logger for ParserLogger{
    fn new(file_name: String, code: String) -> ParserLogger{
        ParserLogger{ file_name, source: Source::from(code), errors: vec![] }
    }

    fn log_error(&self, message: &str, span: (usize, (usize, usize))) {
        let (line_i, (row_start, row_end)) = span;
        let offset = self.source.line(line_i).expect("Custom Span logic returned wrong line ID").offset();
        Report::build(ReportKind::Error, (self.file_name.as_str(), offset + row_start..offset + row_end))
            .with_message(message)
            .with_label(
                Label::new((self.file_name.as_str(), offset + row_start..offset + row_end))
                    .with_message(message)
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((self.file_name.as_str(), self.source.clone()))
            .unwrap();
    }
}

pub static PARSER_LOGGER: OnceLock<Mutex<ParserLogger>> = OnceLock::new();

pub fn init_parser_logger(file_name: String, code: String) {
    let _ = PARSER_LOGGER.set(Mutex::new(ParserLogger::new(file_name, code)));
}

pub fn global_report_parser_error(error: (ParserErrorType, (usize, (usize, usize)))) {
    if let Some(logger) = PARSER_LOGGER.get() {
        let mut logger = logger.lock().unwrap();
        logger.report_error(error);
    } else {
        eprintln!("Parser Logger not initialized correctly");
    }
}

pub fn global_log_errors() {
    if let Some(logger) = PARSER_LOGGER.get() {
        let logger = logger.lock().unwrap();
        logger.log_errors();
    } else {
        eprintln!("Parser Logger not initialized correctly");
    }
}

pub fn failed_parsing() -> bool {
    if let Some(logger) = PARSER_LOGGER.get() {
        let logger = logger.lock().unwrap();
        logger.failed_parsing()
    } else {
        eprintln!("Parser Logger not initialized correctly");
        false
    }
}