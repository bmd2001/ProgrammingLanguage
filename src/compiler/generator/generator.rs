use std::any::type_name;
use either::Either;
use either::Either::{Left, Right};
use crate::compiler::generator::architecture::TARGET_ARCH;
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignment, NodeArithmeticExpr, NodeArithmeticOperation, NodeScope};
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::generator::arithmetic_instructions::{ArithmeticInstructions};
use crate::compiler::generator::stack_handler::StackHandler;

fn type_name_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

pub struct Generator {
    m_prog: NodeProgram,
    m_output: String,
    m_stack: StackHandler,
    m_stack_size: usize,
    m_num_exponentials: usize,
}

impl Generator {
    pub fn new(prog : NodeProgram) -> Self {
        Generator {m_prog: prog, m_output: "".to_string(), m_stack: StackHandler::new(), m_stack_size: 0, m_num_exponentials: 0}
    }

    pub fn generate_comment(comment: &str) -> String {
        if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            format!("\t// {}\n", comment)
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            format!("\t; {}\n", comment)
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            format!("\t; {}\n", comment)
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            format!("\t; {}\n", comment)
        } else {
            String::new()
        }
    }
    
    pub fn get_out_assembly(& self) -> String {
        self.m_output.clone()
    }
    
    pub fn generate(&mut self){
        self.m_output.clear();
        self.m_output.push_str(TARGET_ARCH.get_program_header());
        let stmts = self.m_prog.stmts.clone();
        for stmt in stmts {
            self.generate_stmt(&stmt);
        }
        if !self.m_output.contains(TARGET_ARCH.get_exit_marker()){
            self.m_output.push_str(&Self::generate_comment("Boiler plate for empty script"));
            self.m_output.push_str(TARGET_ARCH.get_exit_instr());
            self.m_output.push_str("\n");
        }
    }
    
    fn generate_stmt(&mut self, stmt: &NodeStmt) {
        match stmt {
            NodeStmt::Exit(exit) => self.generate_exit(exit),
            NodeStmt::ID(var) => self.generate_id(var),
            NodeStmt::Scope(scope) => self.generate_scope(scope),
        }
    }
    
    fn generate_exit(&mut self, exit: &NodeExit){
        self.m_output.push_str(&Self::generate_comment("Exit call"));
        self.m_output.push_str(&Self::generate_comment(&format!("Exit Code = {}", exit.expr)));
        self.generate_arithmetic_expr(&exit.expr);
        self.pop(TARGET_ARCH.get_exit_reg().to_string());
        self.m_output.push_str("\n\t");
        self.m_output.push_str(TARGET_ARCH.get_exit_instr());
        self.m_output.push_str("\n");
        self.m_output.push_str(&Self::generate_comment("Exit end call"));
    }
    
    fn generate_id(&mut self, var: &NodeVariableAssignment) {
        self.m_output.push_str(&Self::generate_comment("VarAssignment"));
        if let Token::ID {name, ..}  = &var.variable{
            self.m_output.push_str(&Self::generate_comment(&format!("{var}")));
            self.generate_arithmetic_expr(&var.value);
            self.m_stack.add_variable(name.clone(), self.infer_type(&var.value).to_string());
        }
    }
    
    fn generate_scope(&mut self, scope: &NodeScope){
        self.m_stack.increase_scope_depth();
        let stmts = scope.stmts.clone();
        for stmt in stmts {
            self.generate_stmt(&stmt);
        }
        self.m_stack.decrease_scope_depth();
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
                if let Token::Number { value, .. } = token {
                    self.m_output.push_str(&format!("\t{}\n", TARGET_ARCH.get_mov_number_instr(value)));
                    if cfg!(target_arch = "x86_64") {
                        self.push("rax");
                    } else if cfg!(target_arch = "aarch64") {
                        self.push("x0");
                    }
                } else {
                    eprintln!("Wrong Tokenization");
                }
            }
            NodeBaseExpr::ID(token) => {
                if let Token::ID { name, .. } = token {
                    let offset = self.m_stack.get_offset(name.clone());
                    self.m_output.push_str(&Self::generate_comment(&format!("Recuperate {name}'s value from stack\n\t{}", TARGET_ARCH.get_load_variable_instr(offset))));

                    if cfg!(target_arch = "x86_64") {
                        self.push("rax");
                    } else if cfg!(target_arch = "aarch64") {
                        self.push("x0");
                    }
                } else {
                    eprintln!("Wrong Tokenization");
                }
            }
            NodeBaseExpr::Bool(token) => {
                if let Token::Boolean { value, .. } = token {
                    self.m_output.push_str(&format!("\t{}\n", TARGET_ARCH.get_mov_boolean_instr(*value)));
                    if cfg!(target_arch = "x86_64") {
                        self.push("rax");
                    } else if cfg!(target_arch = "aarch64") {
                        self.push("x0");
                    }
                } else {
                    eprintln!("Wrong Tokenization");
                }
            }
        }
    }

    //TODO: The multiple similar lines in this method can be refactored by calling a single function that handles everything by accessing the expression
    fn generate_arithmetic_op(&mut self, expr: &NodeArithmeticOperation) {
        let map = ArithmeticInstructions::new();
        match expr.clone().op{
            Operator::Plus { .. } => {
                let instr_data = map.get(&"Addition".to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Addition", instr_data);
            }
            Operator::Minus { .. } => {
                let instr_data = map.get(&"Subtraction".to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Subtraction" , instr_data);
            }
            Operator::Multiplication { .. }=> {
                let instr_data = map.get(&"Multiplication".to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Multiplication" , instr_data);
            }
            Operator::Division { .. } => {
                let instr_data = map.get(&"Division".to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Division" , instr_data);
            },
            Operator::Exponent { .. } => {
                let instr_data = map.get(&"Exponentiation".to_string()).unwrap();
                self.process_binary_operation(expr.clone().rhs, expr.clone().lhs, "Exponentiation" , instr_data);
            }
            Operator::Modulus { .. } => {
                let instr_data = map.get(&"Modulo".to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, "Modulo" , instr_data);
            }
            Operator::Not { .. } => {
                let instr_data = map.get(&"Not".to_string()).unwrap();
                self.process_unary_operation(expr.clone().lhs, instr_data);
            }
            Operator::And { .. } | Operator::Or { .. } | Operator::Xor { .. } => {
                if let (Some(lhs_expr), Some(rhs_expr)) = (Self::extract_expr(&expr.lhs), Self::extract_expr(&expr.rhs)) {
                    if let Err(err) = self.type_check_logical_operands(&lhs_expr, &rhs_expr) {
                        eprintln!("Error: {}", err);
                        return;
                    }
                } else {
                    eprintln!("Error: Missing operand for logical operator");
                    return;
                }

                let op_str = match expr.op {
                    Operator::And { .. } => "And",
                    Operator::Or { .. }  => "Or",
                    Operator::Xor { .. } => "Xor",
                    _ => unreachable!(),
                };

                let instr_data = map.get(&op_str.to_string()).unwrap();
                self.process_binary_operation(expr.clone().lhs, expr.clone().rhs, op_str, instr_data);
            }
            _ => {}
        }
    }

    fn process_unary_operation(
        &mut self,
        operand: Either<Box<NodeArithmeticOperation>, NodeBaseExpr>,
        instruction_data: &((String, String), String, Vec<String>)
    ) {
        self.process_operand(operand);

        let acc_reg = if cfg!(target_arch = "aarch64") {
            "x0"
        } else {
            "rax" // default fallback
        };

        self.pop(acc_reg.to_string());

        let instr_code = instruction_data.2.join("\n\t");
        self.m_output.push_str("\t");
        self.m_output.push_str(&instr_code);
        self.m_output.push_str("\n");

        self.push(acc_reg);
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

    fn process_operand(
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

    fn push_pop(&mut self, pop_regs: (String, String), res_reg: &str, instruction: &str){
        self.pop(pop_regs.1);
        self.pop(pop_regs.0);
        self.m_output.push_str(instruction);
        self.push(res_reg);
    }
    
    fn push(&mut self, reg: &str) {
        if cfg!(target_arch = "x86_64") {
            self.m_output.push_str(&format!("\tpush {}\n", reg));
        } else if cfg!(target_arch = "aarch64") {
            if cfg!(target_os = "linux") {
                self.m_output.push_str(&format!("\tsub sp, sp, #8\n\tstr {}, [sp]\n", reg));
            } else {
                self.m_output.push_str(&format!("\tsub sp, sp, #8\n\tstr {}, [sp]\n", reg));
            }
        }
        self.m_stack_size += 1;
    }
    
    fn pop(&mut self, reg: String) {
        if cfg!(target_arch = "x86_64") {
            self.m_output.push_str(&format!("\tpop {}\n", reg));
        } else if cfg!(target_arch = "aarch64") {
            if cfg!(target_os = "linux") {
                self.m_output.push_str(&format!("\tldr {}, [sp]\n\tadd sp, sp, #8\n", reg));
            } else {
                self.m_output.push_str(&format!("\tldr {}, [sp]\n\tadd sp, sp, #8\n", reg));
            }
        }
        self.m_stack_size -= 1;
    }    
    
    fn generate_exponential_labels(&mut self) -> (String, String){
        let result = (format!("exponential{}", self.m_num_exponentials), format!("exp_done{}", self.m_num_exponentials));
        self.m_num_exponentials += 1;
        result
    }

    fn extract_expr(e: &Either<Box<NodeArithmeticOperation>, NodeBaseExpr>) -> Option<NodeArithmeticExpr> {
        match e {
            Right(base) => Some(NodeArithmeticExpr::Base(base.clone())),
            Left(op) => Some(NodeArithmeticExpr::Operation((**op).clone())),
        }
    }

    fn type_check_logical_operands(&self, lhs: &NodeArithmeticExpr, rhs: &NodeArithmeticExpr) -> Result<(), String> {
        if self.infer_type(lhs) == "bool" && self.infer_type(rhs) == "bool" {
            Ok(())
        } else {
            Err("Logical operators can only be applied to booleans".to_string())
        }
    }

    fn infer_type(&self, expr: &NodeArithmeticExpr) -> &str {
        match expr {
            NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)) => "bool",
            NodeArithmeticExpr::Base(NodeBaseExpr::Num(_)) => "num",
            NodeArithmeticExpr::Base(NodeBaseExpr::ID(_)) => "unknown",
            NodeArithmeticExpr::Operation(_) => "unknown",
        }
    }

}