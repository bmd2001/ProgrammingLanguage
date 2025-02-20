use super::nodes::{
    NodeArithmeticExpr,
    NodeArithmeticOperation,
    NodeBaseExpr,
    NodeExit,
    NodeProgram,
    NodeScope,
    NodeStmt,
    NodeVariableAssignment,
};
use super::parser_logger::{ParserErrorType, ParserLogger};
use crate::compiler::logger::Logger;
use crate::compiler::tokenizer::{Operator, Token};
use either::{Either, Left, Right};
use std::collections::VecDeque;

pub struct Parser {
    m_tokens: Vec<Token>, // Add lifetime annotation
    m_index: usize,
    m_logger: ParserLogger,
    m_errors: Vec<(ParserErrorType, (usize, (usize, usize)))>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file_name: String, text: String) -> Self {
        Parser { m_tokens: tokens , m_index: 0 , m_logger: ParserLogger::new(file_name, text), m_errors: Vec::new()}
    }

    pub fn parse(&mut self) -> Option<NodeProgram>{
        let mut prog = NodeProgram { stmts: Vec::new() };
        while let Some(..) = self.peek(0) {
            if !self.err_token_present() {
                match self.parse_stmt() {
                    Some(stmt) => {
                        prog.stmts.push(stmt);
                    },
                    None => {}
                }
            }
            self.advance_next_stmt(true);
        }
        if self.m_errors.len() > 0 {
            self.m_logger.log_errors(self.m_errors.clone());
            None
        } else { Some(prog) }
    }
    
    fn err_token_present(&mut self) -> bool{
        let mut offset = 0;
        while !matches!(self.peek(offset), None) && !matches!(self.peek(offset), Some(Token::NewLine {..})){
            if matches!(self.peek(offset), Some(Token::Err { .. })){
                self.report_error(ParserErrorType::ErrUnexpectedToken, Some(&self.peek(offset).unwrap().clone()));
                return true;
            }
            offset += 1;
        }
        false
    }

    fn parse_stmt(&mut self) -> Option<NodeStmt> {
        let prev_len = self.m_errors.len();
        if let Some(exit_node) = self.parse_exit(){
            Some(NodeStmt::Exit(exit_node))
        }
        else if let Some(variable_assignment) = self.parse_variable_assignment(){
            Some(NodeStmt::ID(variable_assignment))
        }
        else if let Some(scope_node) = self.parse_scope(){ 
            Some(NodeStmt::Scope(scope_node))
        }
        else if prev_len == self.m_errors.len(){
            let token = self.peek(0).unwrap();
            self.report_error(ParserErrorType::ErrInvalidStatement, Some(&token.clone()));
            return None;
        }
        else { None }
    }
    
    fn parse_exit(&mut self) -> Option<NodeExit>{
        // Check if the first token is 'exit'
        if !matches!(self.peek(0), Some(Token::Exit { .. })) {
            return None;
        }
        // Check if the second token is an opening parenthesis
        if !matches!(self.peek(1), Some(Token::OpenBracket { .. })) {
            let token = self.peek(0).unwrap();
            self.report_error(ParserErrorType::ErrExitOpenBracketMissing, Some(&token.clone()));
            return None;
        }
        // Advance past 'exit' and '(' tokens
        self.advance(2);

        // Parse the arithmetic expression
        let expr = self.parse_arithmetic_expr();

        // Check for closing parenthesis
        if !matches!(self.peek(0), Some(Token::ClosedBracket {..})) {
            self.report_error(ParserErrorType::ErrExitClosedBracketMissing, None);
            return None;
        }

        // Advance past the closing parenthesis
        self.advance(1);

        // Return the parsed NodeExit
        match expr{
            Some(Left(operation)) => {Some(NodeExit { expr: NodeArithmeticExpr::Operation(*operation) })}
            Some(Right(base)) => {Some(NodeExit { expr: NodeArithmeticExpr::Base(base) })}
            None => {None}
        }
    }

    fn type_check_logical_operands(&self, lhs: &NodeArithmeticExpr, rhs: &NodeArithmeticExpr) -> Result<(), String> {
        // A helper function to check if an expression is a boolean literal.
        fn is_boolean(expr: &NodeArithmeticExpr) -> bool {
            match expr {
                NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)) => true,
                _ => false,
            }
        }
        if is_boolean(lhs) && is_boolean(rhs) {
            Ok(())
        } else {
            Err("Logical operators can only be applied to booleans".to_string())
        }
    }
    
    fn parse_variable_assignment(&mut self) -> Option<NodeVariableAssignment>{
        if let Some(tokens) = self.peek_range(3, true){
            return match &tokens[..2] {
                [
                ref id @ Token::ID { .. },           // First token: Identifier
                Token::Equals { .. },            // Second token: Equals
                ] => {
                    self.advance_skip_tokens(2, true, |token| matches!(token, Some(Token::WhiteSpace {..})));
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

    fn parse_arithmetic_expr(&mut self) -> Option<Either<Box<NodeArithmeticOperation>, NodeBaseExpr>> {
        let polish = self.create_reverse_polish_expr();
        let mut expr_stack: Vec<NodeArithmeticExpr> = Vec::new();
        if let Some(p) = polish{
            for token in p{
                match token {
                    Token::ID { .. } => {
                        expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::ID(token.clone())));
                    },
                    Token::Number { .. } => {
                        expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::Num(token.clone())));
                    },
                    Token::Boolean { .. } => {
                        expr_stack.push(NodeArithmeticExpr::Base(NodeBaseExpr::Bool(token.clone())));
                    },
                    Token::Operator(ref op_token) => {
                        let rhs = expr_stack.pop();
                        let lhs = expr_stack.pop();

                        if lhs.is_none() || rhs.is_none() {
                            self.report_error(ParserErrorType::ErrMissingOperand, Some(&token));
                            return None;
                        }

                        // For logical operators, make sure that both operands are booleans.
                        match op_token {
                            Operator::And { .. } | Operator::Or { .. } | Operator::Xor { .. } => {
                                if let (Some(ref lhs_expr), Some(ref rhs_expr)) = (lhs.as_ref(), rhs.as_ref()) {
                                    if let Err(err) = self.type_check_logical_operands(lhs_expr, rhs_expr) {
                                        self.report_error(ParserErrorType::ErrTypeMismatch, Some(&token));
                                        return None;
                                    }
                                }
                            },
                            _ => {}
                        }

                        // Helper function to construct the operation
                        let create_operation = |lhs: NodeArithmeticExpr, rhs: NodeArithmeticExpr| {

                            let lhs_node = match lhs {
                                NodeArithmeticExpr::Base(base) => Right(base),
                                NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
                            };
                            let rhs_node = match rhs {
                                NodeArithmeticExpr::Base(base) => Right(base),
                                NodeArithmeticExpr::Operation(operation) => Left(Box::new(operation))
                            };
                            NodeArithmeticExpr::Operation(NodeArithmeticOperation {
                                lhs: lhs_node,
                                rhs: rhs_node,
                                op: token,
                            })
                        };
                        
                        if matches!(rhs, None) || matches!(lhs, None) {
                            return None;
                        }

                        let operation_node = create_operation(lhs.unwrap(), rhs.unwrap());
                        expr_stack.push(operation_node);
                    }
                    _ => {
                        self.report_error(ParserErrorType::ErrUnexpectedToken, None);
                        return None;
                    }
                }
            }
            match expr_stack.pop(){
                Some(NodeArithmeticExpr::Base(base)) => {Some(Right(base))}
                Some(NodeArithmeticExpr::Operation(op)) => {Some(Left(Box::new(op)))}
                None => {None},
            }
        } else { 
            None
        }
        
        
    }

    fn create_reverse_polish_expr(&mut self) -> Option<VecDeque<Token>>{
        let mut stack: Vec<Operator> = Vec::new();
        let mut polish: VecDeque<Token> = VecDeque::new();
        while let Some(token) = self.peek(0) {
            match token{
                Token::ID { .. } | Token::Number { .. } | Token::Boolean { .. } => {
                    polish.push_back(token.clone());
                },
                Token::Operator(op) => {
                    match op{
                        Operator::OpenBracket { .. } => {
                            stack.push(*op);
                        }
                        Operator::ClosedBracket { .. } => {
                            while !matches!(stack.last(), Some(Operator::OpenBracket {..})) {
                                if stack.is_empty() {
                                    self.report_error(ParserErrorType::ErrExpressionOpenBracketMissing, Some(&token.clone()));
                                    return None;
                                }
                                let op = stack.pop().unwrap();
                                polish.push_back(Token::Operator(op))
                            }
                            stack.pop();
                        }
                        _ => {
                            while let Some(operator) = stack.pop() {
                                if matches!(operator, Operator::OpenBracket {..}) || (operator.precedence() <= op.clone().precedence() && (operator.precedence() != op.clone().precedence() || op.clone().associativity().eq("Right"))){
                                    stack.push(operator);
                                    break;
                                }
                                polish.push_back(Token::Operator(operator));
                            }
                            stack.push(*op);
                        }
                    }
                },
                Token::NewLine {..} | Token::ClosedBracket {..} => {
                    break;
                }
                _ => {
                    self.report_error(ParserErrorType::ErrUnexpectedToken, Some(&token.clone()));
                },
            }
            self.advance_skip_tokens(1, true, |token| matches!(token, Some(Token::WhiteSpace {..})));
        }
        while let Some(i) = stack.pop(){
            if let Operator::OpenBracket { span } = i{
                self.report_error(ParserErrorType::ErrExpressionClosedBracketMissing, Some(&Token::OpenBracket { span }));
                return None;
            }
            polish.push_back(Token::Operator(i));
        }
        Some(polish)
    }
    
    fn parse_scope(&mut self) -> Option<NodeScope>{
        if !matches!(self.peek(0), Some(Token::OpenCurlyBracket { .. })){
            return None;
        }
        let jump_back = &self.m_tokens[self.m_index].get_span();
        self.advance_next_stmt(true);
        let mut stmts = Vec::new();
        //TODO Rewrite this section (from while to the if after)
        while !matches!(self.peek(0), Some(Token::ClosedCurlyBracket { .. })) && !matches!(self.peek(0), None) {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            if !matches!(self.peek(0), Some(Token::ClosedCurlyBracket { .. })){
                self.advance_next_stmt(true);
            }
        }
        if let Some(Token::ClosedCurlyBracket { .. }) = self.peek(0) {
            self.advance(1);
            return Some(NodeScope { stmts })
        }
        self.report_error(ParserErrorType::ErrScopeClosesCurlyBracketMissing, Some(&Token::OpenCurlyBracket { span: *jump_back }));
        None
    }
    
    fn peek(& self, offset: usize) -> Option<&Token> {
        if self.m_index + offset >= self.m_tokens.len() {
            return None;
        }
        Some(&self.m_tokens[self.m_index + offset])
    }

    fn peek_range(& self, count: usize, avoid_space: bool) -> Option<Vec<Token>> {
        let mut index = self.m_index;
        let mut result = Vec::new();

        if avoid_space {
            while index < self.m_tokens.len() && result.len() < count {
                let token = &self.m_tokens[index];
                if !matches!(*token, Token::WhiteSpace {..}){
                    result.push(token.clone());
                }
                if matches!(*token, Token::NewLine {..}){
                    break
                }
                index += 1;
            }
        } else {
            result = self.m_tokens.get(index..index + count)?.to_vec();
        }
        if result.len() == count && !result.iter().any(|token| matches!(token, Token::NewLine { .. })){
            return Some(result)
        }
        None
    }

    fn advance_next_stmt(&mut self, report: bool){
        let skipping_predicate: fn(Option<&Token>) -> bool =
            |token| matches!(token, Some(Token::WhiteSpace {..})) 
                || matches!(token, Some(Token::NewLine {..}));
        if matches!(self.peek(0), Some(Token::OpenCurlyBracket {..})) {
            self.advance(1);
            self.advance_skip_head(skipping_predicate);
        } else if skipping_predicate(self.peek(0)){
            let whitespace_skip_predicate : fn(Option<&Token>) -> bool = |token| matches!(token, Some(Token::WhiteSpace {..}));
            self.advance_skip_head(whitespace_skip_predicate);
            while !matches!(self.peek(0), Some(Token::NewLine {..}) | None | Some(Token::ClosedCurlyBracket {..})){
                if report{ self.report_error(ParserErrorType::ErrUnexpectedToken, None); }
                self.advance_skip_tokens(1, false, whitespace_skip_predicate);
            }
            self.advance_skip_head(skipping_predicate);
        } else {
            self.advance_skip_tokens(1, false, skipping_predicate);   
        }
    }

    fn advance(&mut self, step: usize){
        self.m_index = usize::min(self.m_index + step, self.m_tokens.len());
    }

    fn advance_skip_head<F>(&mut self, skipping_predicate: F)
    where F: Fn(Option<&Token>) -> bool
    {
        while skipping_predicate(self.peek(0)) {
            self.m_index += 1;
        }
    }

    fn advance_skip_tokens<F>(&mut self, step: usize, skip_tail: bool, skipping_predicate: F)
    where F: Fn(Option<&Token>) -> bool + Copy
    {
        let mut num_valid_chars_passed = 0;
        self.advance_skip_head(skipping_predicate);
        while num_valid_chars_passed != step && self.m_index < self.m_tokens.len(){
            if !skipping_predicate(self.peek(0)){
                num_valid_chars_passed += 1;
            }
            self.m_index += 1;
        }
        if skip_tail{
            self.advance_skip_head(skipping_predicate);
        }
    }

    fn report_error(&mut self, parser_error_type: ParserErrorType, token: Option<&Token>){
        let mut span : (usize, (usize, usize)) = (0, (0, 0));
        match parser_error_type {
            ParserErrorType::ErrInvalidStatement => {
                let (stmt_num, _) = token.unwrap().get_span();
                self.advance_next_stmt(false);
                let (_, (_, stmt_end)) = if matches!(self.m_tokens[self.m_index-1], Token::NewLine{..}){
                    self.m_tokens[self.m_index-2].clone().get_span()
                } else { self.m_tokens[self.m_index-1].clone().get_span() };
                span = (stmt_num, (0, stmt_end+1));
            }
            ParserErrorType::ErrExitOpenBracketMissing => {
                if let Some(Token::Exit {span: (exit_line, (_, exit_end))}) = token{
                    span = (*exit_line, (*exit_end, *exit_end))
                }
            }
            ParserErrorType::ErrExitClosedBracketMissing => {
                let token = self.m_tokens[self.m_index-1].clone();
                span = token.get_span();
            }
            ParserErrorType::ErrUnexpectedToken => {
                span = token.unwrap().get_span();
            }
            ParserErrorType::ErrExpressionOpenBracketMissing => {
                span = token.unwrap().get_span();
            }
            ParserErrorType::ErrExpressionClosedBracketMissing => {
                // TODO check for correct parenthesis mismatching detection
                span = token.unwrap().get_span();
            }
            ParserErrorType::ErrMissingOperand => {
                span = token.unwrap().get_span();
            }
            ParserErrorType::ErrTypeMismatch => {
                span = token.unwrap().get_span();
            },
            ParserErrorType::ErrScopeClosesCurlyBracketMissing => span = token.unwrap().get_span()
        }
        self.m_errors.push((parser_error_type, span));
    }

}