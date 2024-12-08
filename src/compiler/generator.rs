use std::any::type_name;
use std::collections::HashMap;
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignement, NodeArithmeticExpr, NodeArithmeticOperation};
use crate::compiler::tokenizer::{Operator, Token};


fn type_name_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

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
        self.generate_arithmetic_expr(&exit.expr);
        self.m_output.push_str("\tmov rax, 0x2000001\n");
        self.pop("rdi");
        self.m_output.push_str("\tsyscall\n\t; Exit end call\n");
    }
    
    fn generate_id(&mut self, var: &NodeVariableAssignement) {
        self.m_output.push_str("\t; VarAssignement\n");
        if let Token::ID {name, ..}  = &var.variable{
            self.m_output.push_str(&format!("\t; {name} = "));
            self.generate_arithmetic_expr(&var.value);
            self.m_id_names.insert(name.clone(), self.m_stack_size-1);
        }
    }
    
    fn generate_arithmetic_expr(&mut self, expr: &NodeArithmeticExpr){
        match expr {
            NodeArithmeticExpr::Base(base) => {self.generate_base_expr(&base.num, false)}
            NodeArithmeticExpr::Operation(operation) => {self.generate_arithmetic_op(operation)}
        }
    }
    
    fn generate_base_expr(&mut self, p_expr: &NodeBaseExpr, from_math: bool){
        match p_expr {
            NodeBaseExpr::Num(token) => {
                if let Token::Number { value, .. } = token{
                    if !from_math {
                        self.m_output.push_str(&format!("{value}\n"));
                    }
                    self.m_output.push_str(&format!("\tmov rax, {value}\n"));
                    self.push("rax")
                } else {
                    eprintln!("Wrong Tokenization")
                }
            }
            NodeBaseExpr::ID(token) => {
                if let Token::ID { name, .. } = token {
                    if let Some(stack_loc) = self.m_id_names.get(name) {
                        let offset = self.m_stack_size.checked_sub(1 + *(stack_loc)).expect("Stack underflow: m_stack_size is smaller than the location of the variable.");
                        if !from_math {
                            self.m_output.push_str(&format!("{name}\n"));
                        }
                        self.m_output.push_str(&format!("\t; Recuperate {name}'s value from stack\n\tmov rax, [rsp + {}]\n", offset * 8)); // Assuming 8-byte stack slots
                        self.push("rax");
                    } else {
                        eprintln!("Variable {name} not defined");
                    }
                } else {
                    eprintln!("Wrong Tokenization")
                }
            }
        }
    }

    fn generate_arithmetic_op(&mut self, expr: &NodeArithmeticOperation) {
        let mut lhs = expr.clone().lhs.clone();
        let mut lhs_str= String::new();
        match lhs{
            NodeBaseExpr::Num(Token::Number {value, .. }) => {lhs_str = value}
            NodeBaseExpr::ID(Token::ID {name, ..}) => {lhs_str = name}
            _ => {}
        }
        let mut rhs = expr.clone().rhs.clone();
        let mut rhs_str= String::new();
        match rhs{
            NodeBaseExpr::Num(Token::Number {value, .. }) => {rhs_str = value}
            NodeBaseExpr::ID(Token::ID {name, ..}) => {rhs_str = name}
            _ => {}
        }
        match expr.clone().op{
            Token::Operator(op_type) => {
                match op_type{
                    Operator::Plus => {
                        self.m_output.push_str(&format!("{} {} {}\n", lhs_str, op_type, rhs_str));
                        self.m_output.push_str("\t; Addition\n ");
                        self.generate_base_expr(&expr.rhs, true);
                        self.generate_base_expr(&expr.lhs, true);
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tadd rax, rbx\n");
                        self.push("rax");
                    }
                    Operator::Minus => {
                        self.m_output.push_str(&format!("{} {} {}\n", lhs_str, op_type, rhs_str));
                        self.m_output.push_str("\t; Subtraction\n ");
                        self.generate_base_expr(&expr.rhs, true);
                        self.generate_base_expr(&expr.lhs, true);
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tsub rax, rbx\n");
                        self.push("rax");
                    }
                    Operator::Multiplication => {
                        self.m_output.push_str(&format!("{} {} {}\n", lhs_str, op_type, rhs_str));
                        self.m_output.push_str("\t; Multiplication\n ");
                        self.generate_base_expr(&expr.rhs, true);
                        self.generate_base_expr(&expr.lhs, true);
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tmul rbx\n");
                        self.push("rax");
                    }
                    Operator::Division => {
                        self.m_output.push_str(&format!("{} {} {}\n", lhs_str, op_type, rhs_str));
                        self.m_output.push_str("\t; Division\n ");
                        self.generate_base_expr(&expr.rhs, true);
                        self.generate_base_expr(&expr.lhs, true);
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tdiv rbx\n");
                        self.push("rax");
                    }
                }
            }
            _ => {
                eprintln!("{}", format!("Wrong Tokenization. Expecting an operator but got {}", type_name_of(&expr.op)))
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