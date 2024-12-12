use std::any::type_name;
use std::collections::HashMap;
use either::Either::{Left, Right};
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignement, NodeArithmeticExpr, NodeArithmeticOperation};
use crate::compiler::tokenizer::{Operator, Token};


fn type_name_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

pub struct Generator {
    m_prog: NodeProgram,
    m_output: String,
    m_id_names: HashMap<String, usize>,
    m_stack_size: usize,
    num_exponentials: usize
}

impl Generator {
    pub fn new(prog : NodeProgram) -> Self {
        Generator {m_prog: prog, m_output: "".to_string(), m_id_names: HashMap::new(), m_stack_size: 0, num_exponentials: 0 }
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
        self.m_output.push_str(&format!("\t; Exit Code = {}\n", exit.expr));
        self.generate_arithmetic_expr(&exit.expr);
        self.m_output.push_str("\tmov rax, 0x2000001\n");
        self.pop("rdi");
        self.m_output.push_str("\tsyscall\n\t; Exit end call\n");
    }
    
    fn generate_id(&mut self, var: &NodeVariableAssignement) {
        self.m_output.push_str("\t; VarAssignement\n");
        if let Token::ID {name, ..}  = &var.variable{
            self.m_output.push_str(&format!("\t; {var}\n"));
            dbg!(format!("\t; {var}\n"));
            self.generate_arithmetic_expr(&var.value);
            self.m_id_names.insert(name.clone(), self.m_stack_size-1);
        }
    }
    
    fn generate_arithmetic_expr(&mut self, expr: &NodeArithmeticExpr){
        match expr {
            NodeArithmeticExpr::Base(base) => {self.generate_base_expr(&base)}
            NodeArithmeticExpr::Operation(operation) => {self.generate_arithmetic_op(operation)}
        }
    }
    
    fn generate_base_expr(&mut self, p_expr: &NodeBaseExpr){
        match p_expr {
            NodeBaseExpr::Num(token) => {
                if let Token::Number { value, .. } = token{
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
        match expr.clone().op{
            Token::Operator(op_type) => {
                match op_type{
                    Operator::Plus => {
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b); self.m_output.push_str("\t; Addition\n ");}
                            Right(base) => {self.m_output.push_str("\t; Addition\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tadd rax, rbx\n");
                        self.push("rax");
                    }
                    Operator::Minus => {
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b); self.m_output.push_str("\t; Subtraction\n ");}
                            Right(base) => {self.m_output.push_str("\t; Subtraction\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rbx");
                        self.pop("rax");
                        self.m_output.push_str("\tsub rax, rbx\n");
                        self.push("rax");
                    }
                    Operator::Multiplication => {
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b); self.m_output.push_str("\t; Multiplication\n ");}
                            Right(base) => {self.m_output.push_str("\t; Multiplication\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rax");
                        self.pop("rbx");
                        self.m_output.push_str("\tmul rbx\n");
                        self.push("rax");
                    }
                    Operator::Division => {
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);self.m_output.push_str("\t; Division\n ");}
                            Right(base) => {self.m_output.push_str("\t; Division\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rbx");
                        self.pop("rax");
                        self.m_output.push_str("\txor rdx, rdx   ; Clear the remainder register\n\tdiv rbx\n");
                        self.push("rax");
                    },
                    Operator::Exponent => {
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);self.m_output.push_str("\t; Exponentiation\n ");}
                            Right(base) => {self.m_output.push_str("\t; Exponentiation\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rdx");
                        self.pop("rcx");
                        let (exp_label, done_label) = self.generate_exponential_labels();
                        self.m_output.push_str(&*format!("\tmov rax, 1\n{exp_label}:\n\tcmp rcx, 0\n\tje {done_label}\n\timul rax, rdx\n\tdec rcx\n\tjmp {exp_label}\n{done_label}:\n"));
                        self.push("rax");
                    }
                    Operator::Modulus => {
                        match expr.clone().lhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);self.m_output.push_str("\t; Modulus\n ");}
                            Right(base) => {self.m_output.push_str("\t; Modulus\n "); self.generate_base_expr(&base);}
                        }
                        match expr.clone().rhs {
                            Left(b) => {self.generate_arithmetic_expr(&b);}
                            Right(base) => { self.generate_base_expr(&base);}
                        }
                        self.pop("rbx");
                        self.pop("rax");
                        self.m_output.push_str("\txor rdx, rdx   ; Clear the remainder register\n\tdiv rbx\n");
                        self.push("rdx");
                    }
                    _ => {}
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
    
    fn generate_exponential_labels(&mut self) -> (String, String){
        let result = (format!("exponential{}", self.num_exponentials), format!("exp_done{}", self.num_exponentials));
        self.num_exponentials += 1;
        result
    }

}