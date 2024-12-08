use std::collections::HashMap;
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodePrimaryExpr, NodeVariableAssignement, NodeArithmeticExpr};
use crate::compiler::tokenizer::{Operator, Token};

pub struct Generator {
    m_prog: NodeProgram,
    m_output: String,
    m_id_names: HashMap<String, usize>,
    m_stack_size: usize
}

impl Generator {
    pub fn new(prog : NodeProgram) -> Self {
        Generator {m_prog: prog, m_output: "".to_string(), m_id_names: HashMap::new(), m_stack_size: 0}
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
        if !self.m_output.contains("syscall"){
            self.m_output.push_str("\t; Boiler plate for empty script\n\tmov rax, 0x2000001\n\tmov rdi, 0\n\tsyscall\n");
        }
    }
    
    fn generate_stmt(&mut self, stmt: &NodeStmt) {
        match stmt {
            NodeStmt::Exit(exit) => {self.generate_exit(exit);}
            NodeStmt::ID(var) => {self.generate_id(var);}
        }
    }
    
    fn generate_exit(&mut self, exit: &NodeExit){
        self.m_output.push_str("\t; Exit call \n");
        self.generate_primary_expr(&exit.expr);
        self.m_output.push_str("\tmov rax, 0x2000001\n");
        self.pop("rdi");
        self.m_output.push_str("\tsyscall\n\t; Exit end call\n");
    }
    
    fn generate_id(&mut self, var: &NodeVariableAssignement) {
        self.m_output.push_str("\t; VarAssignement\n");
        if let Token::ID {name, ..}  = &var.variable{
            self.m_output.push_str(&format!("\t; {name} \n"));
            self.generate_primary_expr(&var.value);
            let stack_loc = self.m_id_names.len();
            self.m_id_names.insert(name.clone(), stack_loc);
        }
    }
    
    fn generate_primary_expr(&mut self, p_expr: &NodePrimaryExpr){
        match p_expr {
            NodePrimaryExpr::Base(token) => {
                self.generate_base_primary_expr(token, false)
            }
            NodePrimaryExpr::Arithmetic(expr) => {
                self.generate_arithmetic_expr(expr)
            }
        }
    }
    
    fn generate_base_primary_expr(&mut self, token: &Token, from_math: bool){
        match token {
            Token::ID { name, .. } => {
                if let Some(stack_loc) = self.m_id_names.get(name) {
                    let offset = self.m_stack_size.checked_sub(*(stack_loc) + 1).expect("Stack underflow: m_stack_size is smaller than the location of the variable.");
                    self.m_output.push_str(&format!("\t; Recuperate {name}'s value from stack\n\tmov rax, [rsp + {}]\n", offset * 8)); // Assuming 8-byte stack slots
                    self.push("rax");
                } else {
                    eprintln!("Variable {name} not defined");
                }
            }
            Token::Number { value, .. } => {
                self.m_output.push_str(&format!("\tmov rax, {value}\n"));
                self.push("rax");}
            _ => {eprintln!("Generating Base Primary Expression went wrong")}
        }
    }

    fn generate_arithmetic_expr(&mut self, expr: &NodeArithmeticExpr) {
        match expr.op{
            Operator::Plus => {
                self.m_output.push_str("\t; Addition\n ");
                self.generate_base_primary_expr(&expr.rhs, true);
                self.generate_base_primary_expr(&expr.lhs, true);
                self.pop("rax");
                self.pop("rbx");
                self.m_output.push_str("\tadd rax, rbx\n");
                self.push("rax");
            }
            Operator::Minus => {
                self.m_output.push_str("\t; Subtraction\n ");
                self.generate_base_primary_expr(&expr.rhs, true);
                self.generate_base_primary_expr(&expr.lhs, true);
                self.pop("rax");
                self.pop("rbx");
                self.m_output.push_str("\tsub rax, rbx\n");
                self.push("rax");
            }
            Operator::Multiply => {
                self.m_output.push_str("\t; Multiplication\n ");
                self.generate_base_primary_expr(&expr.rhs, true);
                self.generate_base_primary_expr(&expr.lhs, true);
                self.pop("rax");
                self.pop("rbx");
                self.m_output.push_str("\tmul rbx\n");
                self.push("rax");
            }
        }
    }
    
    fn push(&mut self, reg: &str) {
        self.m_output.push_str(format!("\tpush {reg}\n").as_str());
        self.m_stack_size += 1;
    }
    
    fn pop(&mut self, reg: &str) {
        self.m_output.push_str(format!("\tpop {reg}\n").as_str());
        self.m_stack_size -= 1;
    }
    
    fn comment_arithemtic_operation(self, expr: NodeArithmeticExpr){
        
    }
    
}