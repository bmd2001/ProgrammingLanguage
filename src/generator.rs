use crate::parser::{NodeProgram, NodeStmt, NodeExit, NodePrimaryExpr};
use crate::tokenizer::Token;

pub struct Generator {
    m_prog: NodeProgram,
    m_output: String
}

impl Generator {
    pub fn new(prog : NodeProgram) -> Self {
        Generator {m_prog: prog, m_output: "".to_string()}
    }
    
    pub fn get_out_assembly(& self) -> String {
        self.m_output.clone()
    }
    
    pub fn generate(&mut self){
        self.m_output.clear();
        self.m_output.push_str("global _start\n_start:\n");
        let stmts = self.m_prog.stmts.clone();
        for stmt in stmts {
            self.generate_stmt(&stmt);
        }
        self.m_output.push_str("\t; Boiler plate for empty script\n\tmov rax, 0x2000001\n\tmov rdi, 0\n\tsyscall\n");
    }
    
    fn generate_stmt(&mut self, stmt: &NodeStmt) {
        self.generate_exit(&stmt.stmt);
    }
    
    fn generate_exit(&mut self, exit: &NodeExit){
        self.m_output.push_str("\t; Exit call\n");
        self.generate_primary_expr(&exit.expr);
        self.m_output.push_str("\tmov rax, 0x2000001\n");
        self.pop("rdi");
        self.m_output.push_str("\tsyscall\n\t; Exit end call\n");
    }
    
    fn generate_primary_expr(&mut self, p_expr: &NodePrimaryExpr){
        if let Token::Number { value, .. } = &p_expr.token {
            self.m_output.push_str(&format!("\tmov rax, {}\n", value));
            self.push("rax");
        }
    }
    
    fn push(&mut self, reg: &str) {
        self.m_output.push_str(format!("\t; Push on stack\n\tpush {reg}\n").as_str());
    }
    
    fn pop(&mut self, reg: &str) {
        self.m_output.push_str(format!("\t; Pop off stack\n\tpop {reg}\n").as_str());
    }
}