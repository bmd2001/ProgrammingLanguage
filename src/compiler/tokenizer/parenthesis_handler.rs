use super::operator::Operator;
use super::token::Token;
use crate::compiler::span::Span;

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

    pub fn emit_bracket_token(&mut self, span: Span , open_bracket: bool) -> Token{
        if open_bracket{ self.handle_open_bracket(span) }
        else { self.handle_closed_bracket(span) }
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


#[cfg(test)]
mod test_parenthesis_handler{
    use super::*;
    
    #[test]
    fn test_init(){
        let handler = ParenthesisHandler::new();
        assert!(!handler.m_function_call);
        assert_eq!(handler.m_bracket_depth, 0);
    }
    
    #[test]
    fn test_activation(){
        let mut handler = ParenthesisHandler::new();
        handler.activate_function_detector();
        assert!(handler.m_function_call);
        assert_eq!(handler.m_bracket_depth, 0);
    }
    
    #[test]
    fn test_deactivation(){
        let mut handler = ParenthesisHandler::new();
        handler.activate_function_detector();
        handler.deactivate_function_detector();
        assert!(!handler.m_function_call);
        assert_eq!(handler.m_bracket_depth, 0);
    }
    
    #[test]
    fn test_open_bracket_emittance(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(0, 0, 0);
        let expected_res = Token::Operator(Operator::OpenBracket {span});
        assert_eq!(expected_res, handler.emit_bracket_token(span, true));
    }

    #[test]
    fn test_closed_bracket_emittance(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(0, 0, 0);
        let expected_res = Token::Operator(Operator::ClosedBracket {span});
        assert_eq!(expected_res, handler.emit_bracket_token(span, false));
    }
    
    #[test]
    fn test_function_call_open_bracket(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(0, 0, 0);
        let expected_res = Token::OpenBracket {span};
        
        handler.activate_function_detector();
        assert_eq!(expected_res, handler.emit_bracket_token(span, true));
        assert_eq!(handler.m_bracket_depth, 1);
    }

    #[test]
    fn test_function_call_closed_bracket_depth_0(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(0, 0, 0);
        let expected_res = Token::ClosedBracket {span};
        
        handler.activate_function_detector();
        assert_eq!(expected_res, handler.emit_bracket_token(span, false));
        assert_eq!(handler.m_bracket_depth, 0);
    }
    
    #[test]
    fn test_function_call_closed_bracket_depth_1(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(0, 0, 0);
        let expected_res = Token::ClosedBracket {span};
        
        handler.activate_function_detector();
        handler.emit_bracket_token(span, true);
        let res = handler.emit_bracket_token(span, false);
        assert_eq!(res, expected_res);
        assert!(!handler.m_function_call);
    }
    
    #[test]
    fn test_function_call_nested_open_brackets() {
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(1, 1, 1);
        let expected_res = Token::Operator(Operator::OpenBracket { span });
        
        handler.activate_function_detector();
        handler.emit_bracket_token(span, true);
        let res = handler.emit_bracket_token(span, true);
        assert_eq!(res, expected_res);
        assert_eq!(handler.m_bracket_depth, 2);
    }
    
    #[test]
    fn test_function_call_nested_brackets(){
        let mut handler = ParenthesisHandler::new();
        let span = Span::new(1, 1, 1);
        let expected_res = Token::Operator(Operator::ClosedBracket {span});
        
        handler.activate_function_detector();
        handler.emit_bracket_token(span, true);
        handler.emit_bracket_token(span, true);
        let res = handler.emit_bracket_token(span, false);
        assert_eq!(res, expected_res);
        assert_eq!(handler.m_bracket_depth, 1);
        assert!(handler.m_function_call);
    }
}