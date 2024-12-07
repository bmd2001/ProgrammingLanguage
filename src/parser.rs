use crate::tokenizer::Token;

pub struct Parser {
    m_tokens: Vec<Token>, // Add lifetime annotation
    m_index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { m_tokens: tokens , m_index: 0 }
    }

    pub fn parse(&mut self) -> Option<NodeProgram>{
        let mut prog = NodeProgram { stmts: Vec::new() };
        while let Some(..) = self.peek(None) {
            if let Some(stmt) = self.parse_stmt(){
                prog.stmts.push(stmt);
            } else {
                eprintln!("The parsed token wasn't a correct stmt");
                std::process::exit(1);
            }
        }
        dbg!(prog.stmts.len());
        Some(prog)
    }

    fn parse_stmt(&mut self) -> Option<NodeStmt> {
        if let Some(exit_node) = self.parse_exit(){
            return Some(NodeStmt::Exit(exit_node));
        } 
        else if let Some(var) = self.parse_variable_assignement(){
            return Some(NodeStmt::ID(var));
        }
        None
    }
    
    fn parse_exit(&mut self) -> Option<NodeExit>{
        let first_is_exit = matches!(self.peek(None), Some(Token::Exit { span: _ }));
        let second_is_oparen = matches!(self.peek(Some(1)), Some(Token::OpenParen));
        if first_is_exit && second_is_oparen {
            self.advance(2);
            if let Some(expr) = self.parse_primary_expr() {
                self.advance(1);
                let last_is_cparen = matches!(self.peek(None), Some(Token::CloseParen));
                return if last_is_cparen {
                    self.advance(1);
                    return Some(NodeExit { expr })
                } else {
                    eprintln!("Error: Final ')' is missing.");
                    None
                }
            }
        }
        None
    }
    
    fn parse_variable_assignement(&mut self) -> Option<NodeVariableAssignement>{
        if let Some(tokens) = self.peek_range(5){
            match (&tokens[0], &tokens[1], &tokens[2], &tokens[3], &tokens[4]) {
                (
                    Token::ID { name, span },           // First token: Identifier
                    Token::WhiteSpace{},
                    Token::Equals { .. },            // Second token: Equals
                    Token::WhiteSpace{},
                    Token::Number { value, span: number_span },     // Third token: Number
                ) => {
                    self.advance(5);
                    Some(NodeVariableAssignement {variable: Token::ID { name: name.clone(), span: *span }, value: Token::Number { value: value.clone(), span: *number_span }})
                }
                _ => None
            }
        } else { None }
    }

    fn parse_primary_expr(&mut self) -> Option<NodePrimaryExpr> {
        let number_condition = matches!(self.peek(None), Some(Token::Number {..}));
        let id_condition = matches!(self.peek(None), Some(Token::ID {..}));
        if number_condition || id_condition {
            let token = self.peek(None).unwrap();
            return Some(NodePrimaryExpr {
                token: token.clone()
            });
        }
        None
    }

    fn peek(& self, offset: Option<usize>) -> Option<&Token> {
        let off = offset.unwrap_or(0);
        if self.m_index + off >= self.m_tokens.len() {
            return None;
        }
        Some(&self.m_tokens[self.m_index + off])
    }

    fn peek_range(& self, count: usize) -> Option<Vec<Token>> {
        if self.m_index+count-1 >= self.m_tokens.len() {
            return None
        }
        Some(self.m_tokens[self.m_index..self.m_index + count].to_owned())
    }
    

    fn advance(&mut self, step: usize) {
        self.m_index = usize::min(self.m_index + step, self.m_tokens.len());
    }
}

pub struct NodeProgram{
    pub(crate) stmts: Vec<NodeStmt>
}

#[derive(Clone)]
pub(crate) enum NodeStmt {
    Exit(NodeExit),
    ID(NodeVariableAssignement)
}

#[derive(Clone)]
pub(crate) struct NodeExit {
    pub(crate) expr: NodePrimaryExpr
}

#[derive(Clone)]
pub struct NodeVariableAssignement{
    pub(crate) variable: Token,
    pub(crate) value: Token
}

#[derive(Clone)]
pub(crate) struct NodePrimaryExpr {
    pub(crate) token: Token
}

#[derive(Clone)]
enum Either<L, R> {
    Left(L),
    Right(R),
}