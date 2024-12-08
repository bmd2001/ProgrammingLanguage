use crate::compiler::tokenizer::Token;

pub struct Parser {
    m_tokens: Vec<Token>, // Add lifetime annotation
    m_index: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { m_tokens: tokens , m_index: 0 }
    }

    pub fn parse(&mut self) -> Result<NodeProgram, String>{
        let mut prog = NodeProgram { stmts: Vec::new() };
        while let Some(..) = self.peek(None) {
            match self.parse_stmt() {
                Ok(stmt) => prog.stmts.push(stmt),
                Err(e) => return Err(format!("The Statement wasn't parsed correctly with the following errors:\n{e}"))
            }
        }
        dbg!(prog.stmts.len());
        Ok(prog)
    }

    fn parse_stmt(&mut self) -> Result<NodeStmt, String> {
        match self.parse_exit() {
            Ok(exit_node) => Ok(NodeStmt::Exit(exit_node)),
            Err(exit_err) => {
                match self.parse_variable_assignement() {
                    Ok(var) => Ok(NodeStmt::ID(var)),
                    Err(var_err) => {
                        Err(format!(
                            "Failed to parse statement:\n- Exit Error: {}\n- Variable Assignment Error: {}",
                            exit_err, var_err
                        ))
                    }
                }
            }
        }
    }
    
    fn parse_exit(&mut self) -> Result<NodeExit, String>{
        let first_is_exit = matches!(self.peek(None), Some(Token::Exit { span: _ }));
        let second_is_oparen = matches!(self.peek(Some(1)), Some(Token::OpenParen));
        if first_is_exit && second_is_oparen {
            self.advance(2);
            let expr = self.parse_primary_expr().map_err(|e| format!("Invalid expression in 'exit': {}", e))?;
            let last_is_cparen = matches!(self.peek(None), Some(Token::CloseParen));
            if last_is_cparen {
                self.advance(1);
                return Ok(NodeExit { expr });
            } else {
                return Err("Error: Final ')' is missing.".to_string());
            }
        }
        Err("Invalid exit.".to_string())
    }
    
    fn parse_variable_assignement(&mut self) -> Result<NodeVariableAssignement, String>{
        if let Some(tokens) = self.peek_range(5){
            match (&tokens[0], &tokens[1], &tokens[2], &tokens[3]) {
                (
                    Token::ID { name, span },           // First token: Identifier
                    Token::WhiteSpace{},
                    Token::Equals { .. },            // Second token: Equals
                    Token::WhiteSpace{},
                ) => {
                    
                    self.advance(4);
                    return match self.parse_primary_expr() {
                        Ok(expr) => {
                            Ok(NodeVariableAssignement {
                                variable: Token::ID { name: name.clone(), span: *span },
                                value: expr,  // The parsed value as a PrimaryExpr
                            })
                        }
                        Err(e) => Err(e), // Handle the error if the last token is not a valid PrimaryExpr
                    }
                }
                _ => {
                    return Err("Invalid syntax for variable assignment. Expected: 'ID = Number'.".to_string());
                }
            }
        }
        Err("Not enough tokens for variable assignment.".to_string())
    }

    fn parse_primary_expr(&mut self) -> Result<NodePrimaryExpr, String> {
        if let Some(token) = self.peek(None).cloned() {
            match token {
                Token::Number { .. } | Token::ID { .. } => {
                    self.advance(1); // Consume the token
                    return Ok(NodePrimaryExpr {
                        token: token.clone(),
                    });
                }
                _ => return Err("Expected a primary expression (Number or ID).".to_string()),
            }
        }
        Err("No token found for primary expression.".to_string())
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
    pub(crate) value: NodePrimaryExpr
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
