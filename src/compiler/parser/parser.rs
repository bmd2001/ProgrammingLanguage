use super::nodes::{NodeProgram};
use super::parser_logger::{ParserLogger};
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
        let mut stmts = Vec::new();
        while self.m_token_stream.peek(0).is_some() {
            let mut stmt_factory = StatementFactory::new(&mut self.m_token_stream, self.m_logger.clone());
            stmt_factory.create(&mut stmts);
            self.m_token_stream.advance_stmt(true);
        }
        if self.flush_errors() {
            None
        } else {
            let prog = NodeProgram { stmts };
            Some(prog)
        }
    }
    
    fn flush_errors(&mut self) -> bool{
        if self.m_logger.lock().is_ok_and(|logger| logger.failed_parsing()) {
            self.m_logger.lock().unwrap().report_errors();
            return true
        }
        false
    }

}



#[cfg(test)]
mod test_parser{
    use crate::compiler::logger::Logger;
    use crate::compiler::parser::{NodeArithmeticExpr, NodeBaseExpr, NodeExit, NodeStmt, NodeVariableAssignment};
    use crate::compiler::span::Span;
    use super::*;
    
    fn create_parser(tokens: Vec<Token>) -> Parser{
        let logger = Arc::new(Mutex::new(ParserLogger::new("".to_string(), "".to_string())));
        Parser::new(tokens, logger)
    }
    
    #[test]
    fn test_parsing(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::ID { name: "x".to_string(), span: dummy_span },
            Token::Equals {span: dummy_span},
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::NewLine { span: dummy_span },
            Token::Exit { span: dummy_span },
            Token::OpenBracket {span: dummy_span},
            Token::Number { value: 1.to_string(), span: dummy_span },
            Token::ClosedBracket {span:dummy_span}
        ];
        
        let mut parser = create_parser(tokens);
        let prog = parser.parse();
        assert!(prog.is_some());
        let node_prog_stmt = prog.unwrap().get_stmts();
        assert!(!node_prog_stmt.is_empty());
        let exp_stmts = vec![
            NodeStmt::ID(
                NodeVariableAssignment {
                    variable: Token::ID { name: "x".to_string(), span: dummy_span },
                    value: NodeArithmeticExpr::Base(NodeBaseExpr::Num(Token::Number { value: 1.to_string(), span: dummy_span })) }
            ),
            NodeStmt::Exit(
                NodeExit{ expr: NodeArithmeticExpr::Base(NodeBaseExpr::Num(Token::Number { value: 1.to_string(), span: dummy_span })) }
            )
        ];
        assert_eq!(node_prog_stmt, exp_stmts);
    }
    
    #[test]
    fn test_wrong_parsing(){
        let dummy_span = Span::new(0, 0, 0);
        let tokens = vec![
            Token::Err {span:dummy_span}
        ];
        let mut parser = create_parser(tokens);
        assert!(parser.parse().is_none());
    }
}