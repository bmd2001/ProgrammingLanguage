use std::sync::{Arc, Mutex};
use either::{Either, Left, Right};
use crate::compiler::parser::{NodeArithmeticExpr, NodeStmt, NodeExit, ParserErrorType, ParserLogger, ExpressionFactory, NodeArithmeticOperation, NodeBaseExpr, NodeScope, NodeVariableAssignment};
use crate::compiler::parser::token_stream::TokenStream;
use crate::compiler::tokenizer::Token;

pub struct StatementFactory<'a>{
    m_token_stream: &'a mut TokenStream,
    m_logger: Arc<Mutex<ParserLogger>>,
}

impl<'a> StatementFactory<'a>{
    pub fn new(m_token_stream: &'a mut TokenStream, m_logger: Arc<Mutex<ParserLogger>>) -> StatementFactory<'a> {
        StatementFactory { m_token_stream, m_logger}
    }
    
    pub fn create(&mut self, stmts: &mut Vec<NodeStmt>){
        if !self.m_token_stream.is_err_in_stmt(){
            let _ = self.parse_stmt().map_or((), |stmt| stmts.push(stmt));
        }
    }

    fn parse_stmt(&mut self) -> Option<NodeStmt> {
        if let Some(exit_node) = self.parse_exit(){
            Some(NodeStmt::Exit(exit_node))
        }
        else if let Some(variable_assignment) = self.parse_variable_assignment(){
            Some(NodeStmt::ID(variable_assignment))
        }
        else if let Some(scope_node) = self.parse_scope(){
            Some(NodeStmt::Scope(scope_node))
        }
        else { None }
    }

    fn parse_exit(&mut self) -> Option<NodeExit>{
        // Check if the first token is 'exit'
        if !matches!(self.m_token_stream.peek(0), Some(Token::Exit { .. })) {
            return None;
        }
        // Check if the second token is an opening parenthesis
        if !matches!(self.m_token_stream.peek(1), Some(Token::OpenBracket { .. })) {
            let token = self.m_token_stream.peek(0).unwrap();
            self.log_error(ParserErrorType::ErrExitOpenBracketMissing, &token);
            return None;
        }
        // Advance past 'exit' and '(' tokens
        self.m_token_stream.advance(2);

        // Parse the arithmetic expression
        let expr = self.parse_arithmetic_expr();

        // Check for closing parenthesis
        if !matches!(self.m_token_stream.peek(0), Some(Token::ClosedBracket {..})) {
            self.log_error(ParserErrorType::ErrExitClosedBracketMissing, &self.m_token_stream.peek_back(1).unwrap());
            return None;
        }

        // Advance past the closing parenthesis
        self.m_token_stream.advance(1);

        // Return the parsed NodeExit
        match expr{
            Some(Left(operation)) => {Some(NodeExit { expr: NodeArithmeticExpr::Operation(*operation) })}
            Some(Right(base)) => {Some(NodeExit { expr: NodeArithmeticExpr::Base(base) })}
            None => {None}
        }
    }

    fn parse_variable_assignment(&mut self) -> Option<NodeVariableAssignment>{
        if let Some(tokens) = self.m_token_stream.peek_range(2, true){
            return match &tokens[..2] {
                [
                ref id @ Token::ID { .. },           // First token: Identifier
                Token::Equals { .. },            // Second token: Equals
                ] => {
                    self.m_token_stream.advance_skip_tokens(2, true, |token| matches!(token, Some(Token::WhiteSpace {..})));
                    match self.parse_arithmetic_expr() {
                        Some(expr) => {
                            Some(NodeVariableAssignment {
                                variable: id.clone(),
                                value: {match expr{
                                    Left(operation) => {NodeArithmeticExpr::Operation(*operation)}
                                    Right(base) => {NodeArithmeticExpr::Base(base)}
                                }
                                },  // The parsed value as a ArithmeticExpr
                            })
                        }
                        None => None, // Handle the error if the last token is not a valid PrimaryExpr
                    }
                }
                _ => {
                    None
                }
            }
        }
        None
    }

    fn parse_scope(&mut self) -> Option<NodeScope>{
        if !matches!(self.m_token_stream.peek(0), Some(Token::OpenCurlyBracket { .. })){
            return None;
        }
        let jump_back = self.m_token_stream.peek(0).unwrap().get_span();
        self.m_token_stream.advance(1);
        self.m_token_stream.advance_stmt(true);
        let mut stmts = Vec::new();
        //TODO Rewrite this section (from while to the if after)
        while !matches!(self.m_token_stream.peek(0), Some(Token::ClosedCurlyBracket { .. })) && !matches!(self.m_token_stream.peek(0), None) {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            if !matches!(self.m_token_stream.peek(0), Some(Token::ClosedCurlyBracket { .. })){
                self.m_token_stream.advance_stmt(true);
            }
        }
        if let Some(Token::ClosedCurlyBracket { .. }) = self.m_token_stream.peek(0) {
            self.m_token_stream.advance(1);
            return Some(NodeScope { stmts })
        }
        self.log_error(ParserErrorType::ErrScopeClosesCurlyBracketMissing, &Token::OpenCurlyBracket { span: jump_back });
        None
    }

    fn parse_arithmetic_expr(&mut self) -> Option<Either<Box<NodeArithmeticOperation>, NodeBaseExpr>> {
        ExpressionFactory::new(&mut self.m_token_stream, self.m_logger.clone()).create()
    }

    fn log_error(&self, error: ParserErrorType, token: &Token){
        let mut logger = self.m_logger.lock().unwrap();
        logger.log_error(error, token);
    }
}