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
        let first_is_exit = matches!(self.peek(None), Some(Token::Exit { span: _ }));
        let second_is_oparen = matches!(self.peek(Some(1)), Some(Token::OpenParen));
        if first_is_exit && second_is_oparen {
            self.advance(2);
            if let Some(expr) = self.parse_primary_expr() {
                self.advance(1);
                let last_is_cparen = matches!(self.peek(None), Some(Token::CloseParen));
                return if last_is_cparen {
                    self.advance(1);
                    Some(NodeStmt {
                        stmt: NodeExit { expr },
                    })
                } else {
                    eprintln!("Error: Final ')' is missing.");
                    None
                }
            }
        }
        None
    }

    fn parse_primary_expr(&mut self) -> Option<NodePrimaryExpr> {
        let condition = matches!(self.peek(None), Some(Token::Number {..}));
        let token = self.peek(None).unwrap();
        if condition {
            return Some(NodePrimaryExpr{
                token: token.clone()
            });
        }
        None
    }

    fn peek(&mut self, offset: Option<usize>) -> Option<&Token> {
        let off = offset.unwrap_or(0);
        if self.m_index + off >= self.m_tokens.len() {
            return None;
        }
        Some(&self.m_tokens[self.m_index + off])
    }

    fn advance(&mut self, step: usize) {
        self.m_index = usize::min(self.m_index + step, self.m_tokens.len());
    }
}

pub struct NodeProgram{
    stmts: Vec<NodeStmt>
}

struct NodeStmt {
    stmt: NodeExit
}

struct NodeExit {
    expr: NodePrimaryExpr
}

struct NodePrimaryExpr {
    token: Token
}