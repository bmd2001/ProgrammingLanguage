use crate::compiler::span::Span;
use super::operator::Operator;
use super::token::Token;

pub struct ParenthesisHandler{
    m_function_call: bool,
    m_bracket_depth: usize
}

impl ParenthesisHandler{
    pub fn new() -> Self {
        ParenthesisHandler { m_function_call: false, m_bracket_depth: 0}
    }

    pub fn activate_function_detector(&mut self){
        self.m_function_call = true;
        self.m_bracket_depth = 0;
    }

    pub fn deactivate_function_detector(&mut self){
        self.m_function_call = false;
        self.m_bracket_depth = 0;
    }

    pub fn emit_bracket_token(&mut self, span: Span , c: char) -> Token{
        if c == '('{
            self.handle_open_bracket(span)
        }
        else if c == ')'{
            self.handle_closed_bracket(span)
        } else {
            panic!("Invalid character passed to Parenthesis Handler")
        }
    }

    fn handle_open_bracket(&mut self, span: Span) -> Token{
        let mut res = Token::Operator(Operator::OpenBracket { span });
        if self.m_function_call{
            if self.m_bracket_depth == 0{
                res = Token::OpenBracket { span };
            }
            self.m_bracket_depth += 1;
        }
        res
    }

    fn handle_closed_bracket(&mut self, span: Span) -> Token {
        let mut res = Token::Operator(Operator::ClosedBracket { span });
        if self.m_function_call {
            if vec![0, 1].contains(&self.m_bracket_depth) {
                res = Token::ClosedBracket { span };
                self.deactivate_function_detector();
            } else {
                self.m_bracket_depth -= 1;
            }
        }
        res
    }
}
