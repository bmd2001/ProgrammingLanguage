use either::Either;
use either::Either::{Left, Right};
use crate::compiler::parser::{NodeProgram, NodeStmt, NodeExit, NodeBaseExpr, NodeVariableAssignment, NodeArithmeticExpr, NodeArithmeticOperation, NodeScope};
use crate::compiler::tokenizer::{Operator, Token};
use crate::compiler::generator::{ArithmeticInstructions, StackHandler, INSTRUCTION_FACTORY};
use crate::utility::{Arch, OS, TARGET_ARCH, TARGET_OS};

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

    pub fn get_out_assembly(& self) -> String {
        self.m_output.clone()
    }
    
    pub fn generate(&mut self){
        self.m_output.clear();
        self.m_output.push_str(INSTRUCTION_FACTORY.get_program_header());
        let stmts = self.m_prog.get_stmts();
        for stmt in stmts {
            self.generate_stmt(&stmt);
        }
        if !self.m_output.contains(INSTRUCTION_FACTORY.get_exit_marker()){
            // TODO This boilerplate is also for a script that doesn't exit
            self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment("Boiler plate for empty script").as_str());
            self.m_output.push_str("\t");
            self.m_output.push_str(INSTRUCTION_FACTORY.get_exit_instr());
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
        self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment("Exit call").as_str());
        self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment(&format!("Exit Code = {}", exit.expr)).as_str());
        self.generate_arithmetic_expr(&exit.expr);
        self.pop(INSTRUCTION_FACTORY.get_exit_reg());
        self.m_output.push_str("\n\t");
        self.m_output.push_str(INSTRUCTION_FACTORY.get_exit_instr());
        self.m_output.push_str("\n");
        self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment("Exit end call").as_str());
    }
    
    fn generate_id(&mut self, var: &NodeVariableAssignment) {
        self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment("VarAssignment").as_str());
        if let Token::ID {name, ..}  = &var.variable{
            self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment(&format!("{var}")).as_str());
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
                    self.m_output.push_str(&format!("\t{}\n", INSTRUCTION_FACTORY.get_mov_number_instr(value)));
                    self.push(TARGET_ARCH.get_base_reg());
                } else {
                    eprintln!("Wrong Tokenization");
                }
            }
            NodeBaseExpr::ID(token) => {
                if let Token::ID { name, .. } = token {
                    let offset = self.m_stack.get_offset(name.clone());
                    self.m_output.push_str(INSTRUCTION_FACTORY.generate_comment(&format!("Recuperate {name}'s value from stack\n\t{}", INSTRUCTION_FACTORY.get_load_variable_instr(offset))).as_str());
                    self.push(TARGET_ARCH.get_base_reg());
                } else {
                    eprintln!("Wrong Tokenization");
                }
            }
            NodeBaseExpr::Bool(token) => {
                if let Token::Boolean { value, .. } = token {
                    self.m_output.push_str(&format!("\t{}\n", INSTRUCTION_FACTORY.get_mov_boolean_instr(*value)));
                    self.push(TARGET_ARCH.get_base_reg());
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

        let acc_reg = TARGET_ARCH.get_base_reg();

        self.pop(acc_reg);

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
        self.pop(&pop_regs.1);
        self.pop(&pop_regs.0);
        self.m_output.push_str(instruction);
        self.push(res_reg);
    }
    
    fn push(&mut self, reg: &str) {
        self.m_output.push_str(&INSTRUCTION_FACTORY.get_push_instr(reg));
        self.m_stack_size += match (TARGET_ARCH, TARGET_OS){
            (Arch::AArch64, OS::MacOS) => 2,
            (Arch::AArch64, OS::Windows) => 2,
            _ => 1
        }
    }
    
    fn pop(&mut self, reg: &str) {
        self.m_output.push_str(&INSTRUCTION_FACTORY.get_pop_instr(reg));
        self.m_stack_size -= match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::MacOS) => 2,
            (Arch::AArch64, OS::Windows) => 2,
            _ => 1
        }
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
    use std::vec::IntoIter;
    use crate::compiler::parser::ResultType;
    use crate::compiler::span::Span;
    use super::*;
    
    fn assert_str_in_out_assembly(gen : &Generator, strs: Vec<&str>) {
        let out = gen.get_out_assembly();
        for str in strs {
            assert!(out.contains(str), "{}", format!("The output doesn't contain \"{}\". The output was: \n{}", str, out))
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
        let exp_instr = INSTRUCTION_FACTORY.get_exponentiation_instr();
        let exp_instr = exp_instr.replace("{exp_label}", &*exp_labels.0);
        let exp_instr = exp_instr.replace("{done_label}", &*exp_labels.1);
        let instrs = vec![
            INSTRUCTION_FACTORY.get_addition_instr().to_string(),
            INSTRUCTION_FACTORY.get_subtraction_instr().to_string(),
            INSTRUCTION_FACTORY.get_multiplication_instr().to_string(),
            INSTRUCTION_FACTORY.get_division_instr().to_string(),
            INSTRUCTION_FACTORY.get_modulo_instr().to_string(),
            exp_instr,
            INSTRUCTION_FACTORY.get_and_instr().to_string(),
            INSTRUCTION_FACTORY.get_or_instr().to_string(),
            INSTRUCTION_FACTORY.get_xor_instr().to_string(),
            INSTRUCTION_FACTORY.get_not_instr().to_string(),
        ];
        zip(ops, instrs)
    }

    #[test]
    fn test_generate_comment() {
        let comment = INSTRUCTION_FACTORY.generate_comment("Test Comment");
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::Linux) => assert_eq!(comment, "\t// Test Comment\n"),
            _ => assert_eq!(comment, "\t; Test Comment\n")
        }
    }

    #[test]
    fn test_push_pop() {
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        
        gen.push(TARGET_ARCH.get_base_reg());
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::MacOS) => {assert_eq!(gen.m_stack_size, 2);}
            (Arch::AArch64, OS::Windows) => {assert_eq!(gen.m_stack_size, 2);}
            _ => {assert_eq!(gen.m_stack_size, 1);}
        }
        gen.pop(TARGET_ARCH.get_base_reg());
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
            INSTRUCTION_FACTORY.get_exit_instr()
        ];
        assert_str_in_out_assembly(&gen, should_contain);
    }
    
    #[test]
    fn test_no_exit(){
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        gen.generate();
        let should_contain = vec![
            "Boiler plate for empty script",
            INSTRUCTION_FACTORY.get_exit_instr()
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
        let push_reg = TARGET_ARCH.get_base_reg();
        let mov_instr = INSTRUCTION_FACTORY.get_mov_number_instr("42");
        let push_instr = match TARGET_ARCH{
            Arch::X86_64 => {format!("\tpush {}\n", push_reg)}
            Arch::AArch64 => {
                // On ARM64, the updated push routine subtracts 16 and then stores the value at offset 8.
                format!("\tsub sp, sp, #16\n\tstr {}, [sp, #8]\n", push_reg)
            }
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
            INSTRUCTION_FACTORY.get_exit_instr()
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
        let reg = TARGET_ARCH.get_base_reg();
        
        // First push
        gen.push(reg);
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::MacOS) => {assert_eq!(gen.m_stack_size, 2);}
            (Arch::AArch64, OS::Windows) => {assert_eq!(gen.m_stack_size, 2);}
            _ => {assert_eq!(gen.m_stack_size, 1);}
        }
        
        // Second push
        gen.push(reg);
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::MacOS) => {assert_eq!(gen.m_stack_size, 4);}
            (Arch::AArch64, OS::Windows) => {assert_eq!(gen.m_stack_size, 4);}
            _ => {assert_eq!(gen.m_stack_size, 2);}
        }
        
        let x86_expected = format!("\tpush {reg}\n");
        let arm_expected = format!("\tsub sp, sp, #16\n\tstr {reg}, [sp, #8]\n");
        let should_contain = match TARGET_ARCH {
            Arch::X86_64 => vec![x86_expected.as_str()],
            Arch::AArch64 => vec![arm_expected.as_str()]
        };
        
        assert_str_in_out_assembly(&gen, should_contain);
    }    

    #[test]
    fn test_pop(){
        let mut gen = Generator::new(NodeProgram { stmts: Vec::new() });
        let reg = TARGET_ARCH.get_base_reg();
        gen.push(reg);
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::MacOS) => {assert_eq!(gen.m_stack_size, 2);}
            (Arch::AArch64, OS::Windows) => {assert_eq!(gen.m_stack_size, 2);}
            _ => {assert_eq!(gen.m_stack_size, 1);}
        }
        gen.pop(reg);
        assert_eq!(gen.m_stack_size, 0);
    
        let x86_expected = format!("\tpop {reg}\n");
        let arm_expected = format!("\tldr {reg}, [sp, #8]\n\tadd sp, sp, #16\n");
        let should_contain = match TARGET_ARCH {
            Arch::X86_64 => {vec![x86_expected.as_str()]}
            Arch::AArch64 => {vec![arm_expected.as_str()]}
        };
        assert_str_in_out_assembly(&gen, should_contain);
    
        // Test that popping when the stack is empty causes a panic.
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let failure = panic::catch_unwind(panic::AssertUnwindSafe(|| gen.pop(reg)));
        panic::set_hook(prev_hook);
        assert!(failure.is_err());
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