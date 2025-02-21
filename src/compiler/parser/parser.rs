use super::nodes::{NodeProgram};
use super::parser_logger::{ParserErrorType, ParserLogger};
use crate::compiler::tokenizer::{Token};
use super::token_stream::TokenStream;
use std::sync::{Arc, Mutex};
use crate::compiler::parser::statement_factory::StatementFactory;

pub struct Parser{ 
    m_token_stream: TokenStream,
    m_logger: Arc<Mutex<ParserLogger>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, m_logger: Arc<Mutex<ParserLogger>>) -> Self {
        // Initialize TokenStream with owned tokens
        Parser {
            m_token_stream: TokenStream::new(tokens.clone(), m_logger.clone()),
            m_logger
        }
    }

    pub fn parse(&mut self) -> Option<NodeProgram> {
        let mut prog = NodeProgram { stmts: Vec::new() };
        while let Some(..) = self.m_token_stream.peek(0) {
            let mut stmt_factory = StatementFactory::new(&mut self.m_token_stream, self.m_logger.clone());
            stmt_factory.create(&mut prog.stmts);
            self.m_token_stream.advance_stmt(true);
        }
        if self.flush_errors() {
            None
        } else { Some(prog) }
    }

    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.test(error, token);
    }
    
    fn flush_errors(&mut self) -> bool{
        if self.m_logger.lock().is_ok_and(|logger| logger.failed_parsing()) {
            self.m_logger.lock().unwrap().report_errors();
            return true
        }
        false
    }

}