use std::any::type_name;
use std::collections::HashMap;
use either::Either;
use either::Either::{Left, Right};
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignement, NodeArithmeticExpr, NodeArithmeticOperation};
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::arithmetic_instructions::{ArithmeticInstructions};


fn type_name_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

pub struct Generator {
    m_prog: NodeProgram,
    m_output: String,
    m_id_names: HashMap<String, usize>,
    m_stack_size: usize,
    num_exponentials: usize,
}

impl Generator {
    pub fn new(prog : NodeProgram) -> Self {
        Generator {m_prog: prog, m_output: "".to_string(), m_id_names: HashMap::new(), m_stack_size: 0, num_exponentials: 0, }
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
            NodeStmt::Exit(exit) => self.generate_exit(exit),
            NodeStmt::ID(var) => self.generate_id(var)
        }
    }
    
    fn generate_exit(&mut self, exit: &NodeExit){
        self.m_output.push_str("\t; Exit call \n");
        self.m_output.push_str(&format!("\t; Exit Code = {}\n", exit.expr));
        self.generate_arithmetic_expr(&exit.expr);
        self.m_output.push_str("\tmov rax, 0x2000001\n");
        self.pop("rdi".to_string());
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
            NodeArithmeticExpr::Base(base) => self.generate_base_expr(&base),
            NodeArithmeticExpr::Operation(operation) => self.generate_arithmetic_op(operation)
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

    //TODO: The multiple similar lines in this method can be refactored by calling a single function that handles everything by accessing the expression
    fn generate_arithmetic_op(&mut self, expr: &NodeArithmeticOperation) {
        let map = ArithmeticInstructions::new();
        match expr.clone().op{
            Token::Operator(op_type) => {
                match op_type{
                    Operator::Plus => {
                        let instr_data = map.get(&"Addition".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Addition", instr_data);
                    }
                    Operator::Minus => {
                        let instr_data = map.get(&"Subtraction".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Subtraction" , instr_data);
                    }
                    Operator::Multiplication => {
                        let instr_data = map.get(&"Multiplication".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Multiplication" , instr_data);
                    }
                    Operator::Division => {
                        let instr_data = map.get(&"Division".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Division" , instr_data);
                    },
                    Operator::Exponent => {
                        let instr_data = map.get(&"Exponentiation".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().rhs, expr.clone().lhs, "Exponentiation" , instr_data);
                    }
                    Operator::Modulus => {
                        let instr_data = map.get(&"Modulo".to_string()).unwrap();
                        self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Modulo" , instr_data);
                    }
                    _ => {}
                }
            }
            _ => {
                eprintln!("{}", format!("Wrong Tokenization. Expecting an operator but got {}", type_name_of(&expr.op)))
            }
        }
    }
    
    fn process_binary_operation(
        &mut self, 
        lhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>, 
        rhs: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>, 
        instruction: &str,
        instruction_data: &((String, String), String, Vec<String>)
    ) {
        self.process_operand(lhs);
        self.process_operand(rhs);
        let (exp_label, done_label) : (Option<String>, Option<String>) = if instruction == "Exponentiation"{
            let (exp, done) = self.generate_exponential_labels();
            (Some(exp), Some(done))
        } else{
            (None, None)
        };
        fn process_instruction(exp_label: Option<String>, done_label: Option<String>, lines: &Vec<String>) -> String {
            let mut res = lines.join("\n\t");
            res.insert_str(0, "\t");
            res.push_str("\n");
            res = res.replace("\t{", "{");
            if let Some(exp) = exp_label{
                if let Some(done) = done_label{
                    res = res.replace("{exp_label}", &exp).replace("{done_label}", &done);
                }
            }
            res
        }
        let ((reg1, reg2), res_reg, instr) = instruction_data;
        self.push_pop((reg1.clone(), reg2.clone()), res_reg, &*process_instruction(exp_label, done_label, instr));
    }

    fn process_operand (
        &mut self,
        operand: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>
    ) {
        match operand {
            Left(b) => {
                self.generate_arithmetic_expr(&NodeArithmeticExpr::Operation(*b));
            },
            Right(base) => {
                self.generate_base_expr(&base);
            }
        }
    }

    fn push_pop (&mut self, pop_regs: (String, String), res_reg: &str, instruction: &str){
        self.pop(pop_regs.1);
        self.pop(pop_regs.0);
        self.m_output.push_str(instruction);
        self.push(res_reg);
    }
    
    fn push(&mut self, reg: &str) {
        self.m_output.push_str(format!("\tpush {reg}\n").as_str());
        self.m_stack_size += 1;
    }
    
    fn pop(&mut self, reg: String) {
        self.m_output.push_str(format!("\tpop {reg}\n").as_str());
        self.m_stack_size -= 1;
    }
    
    fn generate_exponential_labels(&mut self) -> (String, String){
        let result = (format!("exponential{}", self.num_exponentials), format!("exp_done{}", self.num_exponentials));
        self.num_exponentials += 1;
        result
    }

}