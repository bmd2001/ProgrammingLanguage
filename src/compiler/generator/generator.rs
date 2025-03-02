use either::Either;
use either::Either::{Left, Right};
use crate::compiler::generator::architecture::TARGET_ARCH;
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignment, NodeArithmeticExpr, NodeArithmeticOperation, NodeScope};
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::generator::arithmetic_instructions::{ArithmeticInstructions};
use crate::compiler::generator::stack_handler::StackHandler;

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
        let stmts = self.m_prog.get_stmts();
        for stmt in stmts {
            self.generate_stmt(&stmt);
        }
        if !self.m_output.contains(TARGET_ARCH.get_exit_marker()){
            // TODO This boilerplate is also for a script that doesn't exit
            self.m_output.push_str(&Self::generate_comment("Boiler plate for empty script"));
            self.m_output.push_str("\t");
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
            _ => {unreachable!()}
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
            self.m_output.push_str(&format!("\tsub sp, sp, #8\n\tstr {}, [sp]\n", reg));
        }
        self.m_stack_size += 1;
    }
    
    fn pop(&mut self, reg: String) {
        if cfg!(target_arch = "x86_64") {
            self.m_output.push_str(&format!("\tpop {}\n", reg));
        } else if cfg!(target_arch = "aarch64") {
            self.m_output.push_str(&format!("\tldr {}, [sp]\n\tadd sp, sp, #8\n", reg));
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

    fn infer_type<'a>(&'a self, expr: &'a NodeArithmeticExpr) -> &'a str {
        match expr {
            NodeArithmeticExpr::Base(NodeBaseExpr::Bool(_)) => "bool",
            NodeArithmeticExpr::Base(NodeBaseExpr::Num(_)) => "num",
            NodeArithmeticExpr::Base(NodeBaseExpr::ID(_)) => "unknown",
            NodeArithmeticExpr::Operation(NodeArithmeticOperation { result_type, .. }) => result_type.as_str(),
        }
    }
}



#[cfg(test)]
mod test_generator{
    use std::iter::{zip, Zip};
    use std::panic;
    use std::panic::AssertUnwindSafe;
    use std::vec::IntoIter;
    use crate::compiler::parser::ResultType;
    use crate::compiler::span::Span;
    use super::*;
    
    fn assert_str_in_out_assembly(gen : &Generator, strs: Vec<&str>){
        let out = gen.get_out_assembly();
        for str in strs{
            assert!(out.contains(str), "{}", format!("The output doesn't contain \"{}\". The output was: \n{}", str, out))
        }
    }
    
    fn get_correct_reg() -> &'static str{
        if cfg!(target_arch = "x86_64") {
            "rax"
        } else if cfg!(target_arch = "aarch64") {
            "x0"
        } else {
            "rax"
        }
    }
    
    fn create_operations() -> (Vec<NodeStmt>, Vec<String>){
        let dummy_span = Span::new(0, 0, 0);
        let expr1 = NodeBaseExpr::Num(Token::Number { value: "1".to_string(), span: dummy_span });
        let expr2 = NodeBaseExpr::Num(Token::Number { value: "2".to_string(), span: dummy_span });
        let expr3 = NodeBaseExpr::Bool(Token::Boolean { value: true, span: dummy_span });
        let expr4 = NodeBaseExpr::Bool(Token::Boolean { value: false, span: dummy_span }); 
        let mut res_stmts = Vec::new();
        let mut res_exp_out = Vec::new();
        for (op, instr) in get_all_operators(){
            let mut lhs = Right(expr1.clone());
            let mut rhs = Right(expr2.clone());
            let mut result_type = ResultType::Numeric;
            let mut comment = format!("1 {} 2", op);
            match op{
                Operator::And {..} | Operator::Or {..} | Operator::Not {..} | Operator::Xor {..} => {
                    lhs = Right(expr3.clone());
                    rhs = Right(expr4.clone());
                    result_type = ResultType::Boolean;
                    comment = format!("true {} false", op);
                }
                _ => {}
            }
            let operation = NodeArithmeticExpr::Operation(NodeArithmeticOperation{
                lhs,
                rhs,
                op,
                result_type,
            });
            let var = Token::ID { name: "x".to_string(), span: dummy_span };
            let id_assignment_stmt = NodeStmt::ID(NodeVariableAssignment{ variable: var, value: operation.clone() });
            res_stmts.push(id_assignment_stmt);
            res_exp_out.push(comment);
            res_exp_out.push(instr);
        }
        (res_stmts, res_exp_out)
    }
    
    fn get_all_operators() -> Zip<IntoIter<Operator>, IntoIter<String>> {
        let dummy_span = Span::new(0, 0, 0);
        let ops = vec![
            Operator::Plus { span: dummy_span },
            Operator::Minus { span: dummy_span},
            Operator::Multiplication { span: dummy_span },
            Operator::Division { span: dummy_span },
            Operator::Modulus { span: dummy_span },
            Operator::Exponent { span: dummy_span },
            Operator::And { span: dummy_span },
            Operator::Or { span: dummy_span },
            Operator::Xor { span: dummy_span },
            Operator::Not {span: dummy_span}
        ];
        let mut gen = Generator::new(NodeProgram{ stmts: vec![] });
        let exp_labels = gen.generate_exponential_labels();
        let exp_instr = TARGET_ARCH.get_exponentiation_instr();
        let exp_instr = exp_instr.replace("{exp_label}", &*exp_labels.0);
        let exp_instr = exp_instr.replace("{done_label}", &*exp_labels.1);
        let instrs = vec![
            TARGET_ARCH.get_addition_instr().to_string(),
            TARGET_ARCH.get_subtraction_instr().to_string(),
            TARGET_ARCH.get_multiplication_instr().to_string(),
            TARGET_ARCH.get_division_instr().to_string(),
            TARGET_ARCH.get_modulo_instr().to_string(),
            exp_instr,
            TARGET_ARCH.get_and_instr().to_string(),
            TARGET_ARCH.get_or_instr().to_string(),
            TARGET_ARCH.get_xor_instr().to_string(),
            TARGET_ARCH.get_not_instr().to_string(),
        ];
        zip(ops, instrs)
    }

    #[test]
    fn test_generate_comment() {
        if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64"){
            let comment = Generator::generate_comment("Test Comment");
            assert_eq!(comment, "\t// Test Comment\n");
        }
        else {
            let comment = Generator::generate_comment("Test Comment");
            assert_eq!(comment, "\t; Test Comment\n");
        }
    }

    #[test]
    fn test_push_pop() {
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        
        gen.push(get_correct_reg());
        assert_eq!(gen.m_stack_size, 1);
        gen.pop(get_correct_reg().to_string());
        assert_eq!(gen.m_stack_size, 0);
    }

    #[test]
    fn test_generate_exit() {
        let dummy_span = Span::new(0, 0, 0);
        let expr = NodeArithmeticExpr::Base(NodeBaseExpr::Num(Token::Number { value: "42".to_string(), span: dummy_span }));
        let exit_stmt = NodeStmt::Exit(NodeExit { expr });
        let mut gen = Generator::new(NodeProgram { stmts: vec![exit_stmt] });

        gen.generate();
        let should_contain = vec![
            "Exit call",
            "Exit Code = 42",
            TARGET_ARCH.get_exit_instr()
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }
    
    #[test]
    fn test_no_exit(){
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        gen.generate();
        let should_contain = vec![
            "Boiler plate for empty script",
            TARGET_ARCH.get_exit_instr()
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }

    #[test]
    fn test_generate_id(){
        let dummy_span = Span::new(0, 0, 0);
        let expr = NodeArithmeticExpr::Base(NodeBaseExpr::Num(Token::Number { value: "42".to_string(), span: dummy_span }));
        let var = Token::ID { name: "x".to_string(), span: dummy_span };
        let id_assignment_stmt = NodeStmt::ID(NodeVariableAssignment{ variable: var, value: expr });
        let mut gen = Generator::new(NodeProgram { stmts: vec![id_assignment_stmt] });

        gen.generate();
        let push_reg = get_correct_reg();
        let mov_instr = TARGET_ARCH.get_mov_number_instr("42");
        let push_instr = if cfg!(target_arch = "x86_64") {
            format!("\tpush {}\n", push_reg)
        } else {
            format!("\tsub sp, sp, #8\n\tstr {}, [sp]\n", push_reg)
        };
        let should_contain = vec![
            "VarAssignment",
            mov_instr.as_str(),
            push_instr.as_str(),
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }
    
    #[test]
    fn test_generate_scope(){
        let dummy_span = Span::new(0, 0, 0);
        let expr = NodeArithmeticExpr::Base(NodeBaseExpr::Num(Token::Number { value: "42".to_string(), span: dummy_span }));
        let var = Token::ID { name: "x".to_string(), span: dummy_span };
        let id_assignment_stmt = NodeStmt::ID(NodeVariableAssignment{ variable: var, value: expr.clone() });
        let exit_stmt = NodeStmt::Exit(NodeExit { expr });
        let scope_stmt = NodeStmt::Scope(NodeScope{stmts: vec![id_assignment_stmt, exit_stmt]});

        let mut gen = Generator::new(NodeProgram { stmts: vec![scope_stmt] });

        gen.generate();
        let should_contain = vec![
            "VarAssignment",
            "Exit call",
            "Exit Code = 42",
            TARGET_ARCH.get_exit_instr()
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }
    
    #[test]
    fn test_generate_operation_id(){
        let (stmts, should_contain_strs) = create_operations();
        let should_contain = should_contain_strs.iter().map(|string| string.as_str()).collect();
        let mut gen = Generator::new(NodeProgram { stmts });

        gen.generate();
        assert_str_in_out_assembly(&gen, should_contain);
    }
    
    #[test]
    fn test_generate_nested_operation_id(){
        let dummy_span = Span::new(0, 0, 0);
        let expr = Token::Number { value: "42".to_string(), span: dummy_span };
        let var = Token::ID { name: "x".to_string(), span: dummy_span };
        let id_assignment_stmt = NodeStmt::ID(NodeVariableAssignment{ variable: var.clone(), value: NodeArithmeticExpr::Base(NodeBaseExpr::Num(expr.clone()))});
        let nested_expr = NodeArithmeticExpr::Operation(NodeArithmeticOperation{
            lhs: Left(Box::new(NodeArithmeticOperation{
                lhs: Right(NodeBaseExpr::Num(expr.clone())),
                rhs: Right(NodeBaseExpr::ID(var.clone())),
                op: Operator::Plus {span : dummy_span},
                result_type: ResultType::Numeric,
            })),
            rhs: Right(NodeBaseExpr::Num(expr.clone())),
            op: Operator::Plus {span: dummy_span},
            result_type: ResultType::Numeric,
        });
        let id_second_stmt = NodeStmt::ID(NodeVariableAssignment{ variable: var, value: nested_expr });
        let mut gen = Generator::new(NodeProgram { stmts: vec![id_assignment_stmt, id_second_stmt] });
        
        gen.generate();
        let should_contain = vec![
            "VarAssignment",
            "x = (42 + x) + 42"
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }

    #[test]
    fn test_exp_labels(){
        let mut gen = Generator::new(NodeProgram{stmts: Vec::new()});
        assert_eq!(gen.generate_exponential_labels(), ("exponential0".to_string(), "exp_done0".to_string()));
        assert_eq!(gen.m_num_exponentials, 1);
        assert_eq!(gen.generate_exponential_labels(), ("exponential1".to_string(), "exp_done1".to_string()));
        assert_eq!(gen.m_num_exponentials, 2)
    }

    #[test]
    fn test_push(){
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        let reg = get_correct_reg();
        gen.push(reg);
        assert_eq!(gen.m_stack_size, 1);
        gen.push(reg);
        assert_eq!(gen.m_stack_size, 2);
        let x86 = format!("\tpush {reg}\n");
        let arch = format!("\tsub sp, sp, #8\n\tstr {reg}, [sp]\n");
        let should_contain = if cfg!(target_arch = "x86_64") { 
            vec![x86.as_str()]
        } else { vec![arch.as_str()] };
        assert_str_in_out_assembly(&gen, should_contain);
    }

    #[test]
    fn test_pop(){
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        let reg = get_correct_reg();
        gen.push(reg);
        assert_eq!(gen.m_stack_size, 1);
        gen.pop(reg.to_string());
        assert_eq!(gen.m_stack_size, 0);
        let x86 = format!("\tpop {reg}\n");
        let arch = format!("\tldr {reg}, [sp]\n\tadd sp, sp, #8\n");
        let should_contain = if cfg!(target_arch = "x86_64") {
            vec![x86.as_str()]
        } else { vec![arch.as_str()] };
        assert_str_in_out_assembly(&gen, should_contain);
        
        // To prevent rust to print the panic message in the terminal, we replace the panic before calling decrease_scope_depth
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let failure = panic::catch_unwind(AssertUnwindSafe(|| gen.pop(reg.to_string())));
        panic::set_hook(prev_hook);
        assert!(failure.is_err())
    }

    #[test]
    fn test_extract_expr(){
        let dummy_span = Span::new(0, 0, 0);
        let num = NodeBaseExpr::Num(Token::Number { value: 1.to_string(), span: dummy_span });
        let bool = NodeBaseExpr::Bool(Token::Boolean { value: true, span: dummy_span });
        let var = NodeBaseExpr::ID(Token::ID { name: "x".to_string(), span: dummy_span });
        let operation = NodeArithmeticOperation{
            lhs: Right(num),
            rhs: Right(var),
            op: Operator::Plus { span: dummy_span },
            result_type: ResultType::Numeric,
        };
        let extracted_operation = Generator::extract_expr(&Left(Box::new(operation.clone())));
        assert_eq!(extracted_operation, Some(NodeArithmeticExpr::Operation(operation)));

        let extracted_base = Generator::extract_expr(&Right(bool.clone()));
        assert_eq!(extracted_base, Some(NodeArithmeticExpr::Base(bool)));
    }
    
    #[test]
    fn test_infer_type(){
        let gen = Generator::new(NodeProgram{stmts: Vec::new()});
        let dummy_span = Span::new(0, 0, 0);
        let num = NodeBaseExpr::Num(Token::Number { value: 1.to_string(), span: dummy_span });
        let bool = NodeBaseExpr::Bool(Token::Boolean { value: true, span: dummy_span });
        let var = NodeBaseExpr::ID(Token::ID { name: "x".to_string(), span: dummy_span });
        assert_eq!(gen.infer_type(&NodeArithmeticExpr::Base(num.clone())), "num");
        assert_eq!(gen.infer_type(&NodeArithmeticExpr::Base(bool.clone())), "bool");
        assert_eq!(gen.infer_type(&NodeArithmeticExpr::Base(var.clone())), "unknown");
        
        let operation = NodeArithmeticExpr::Operation(NodeArithmeticOperation{
            lhs: Right(num),
            rhs: Right(var),
            op: Operator::Plus { span: dummy_span },
            result_type: ResultType::Numeric,
        });
        assert_eq!(gen.infer_type(&operation), "num");
    }

    #[test]
    fn test_type_check_logical_operands(){
        let gen = Generator::new(NodeProgram{stmts: Vec::new()});
        let dummy_span = Span::new(0, 0, 0);
        let num_base = NodeBaseExpr::Num(Token::Number { value: 1.to_string(), span: dummy_span });
        let bool_base = NodeBaseExpr::Bool(Token::Boolean { value: true, span: dummy_span });
        let var_base = NodeBaseExpr::ID(Token::ID { name: "x".to_string(), span: dummy_span });
        let num = NodeArithmeticExpr::Base(num_base.clone());
        let bool = NodeArithmeticExpr::Base(bool_base.clone());
        let var = NodeArithmeticExpr::Base(var_base.clone());
        let num_operation = NodeArithmeticExpr::Operation(NodeArithmeticOperation{
            lhs: Right(num_base),
            rhs: Right(var_base),
            op: Operator::Plus { span: dummy_span },
            result_type: ResultType::Numeric,
        });
        let bool_operation = NodeArithmeticExpr::Operation(NodeArithmeticOperation{
            lhs: Right(bool_base.clone()),
            rhs: Right(bool_base),
            op: Operator::And {span: dummy_span},
            result_type: ResultType::Boolean,
        });

        let valid1 = gen.type_check_logical_operands(&bool, &bool);
        let valid2 = gen.type_check_logical_operands(&bool_operation, &bool);
        let valid = vec![valid1, valid2];
        for v in valid{
            assert!(v.is_ok());
        }

        let err1 = gen.type_check_logical_operands(&num, &num);
        let err2 = gen.type_check_logical_operands(&var, &var);
        let err3 = gen.type_check_logical_operands(&var, &num);
        let err4 = gen.type_check_logical_operands(&num_operation, &bool);
        let err5 = gen.type_check_logical_operands(&bool_operation, &num);
        let errors = vec![err1, err2, err3, err4, err5];
        for err in errors{
            assert!(err.is_err());
        }
    }
}