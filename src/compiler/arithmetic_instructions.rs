use std::collections::HashMap;
use crate::compiler::architecture::{Arch, TARGET_ARCH};

pub struct ArithmeticInstructions {
    instrs: HashMap<String, ((String, String), String, Vec<String>)>
}
impl ArithmeticInstructions {
    pub fn new() -> Self {
        fn operation(
            reg_lhs: &str,
            reg_rhs: &str,
            result_reg: &str,
            instructions: Vec<&str>,
            ) -> ((String, String), String, Vec<String>) {
                (
                (String::from(reg_lhs), String::from(reg_rhs)),
                String::from(result_reg),
                instructions.into_iter().map(String::from).collect(),
                )
        }

        let (arith_reg_lhs, arith_reg_rhs, arith_result_reg) = match TARGET_ARCH {
            Arch::X86_64 => ("rax", "rbx", "rax"),
            Arch::AArch64 => ("x0", "x1", "x0"),
        };

        let (exp_reg_lhs, exp_reg_rhs, exp_result_reg) = match TARGET_ARCH {
            Arch::X86_64 => ("rcx", "rdx", "rax"),
            Arch::AArch64 => ("x1", "x2", "x0"),
        };

        let modulo_result_reg = match TARGET_ARCH {
            Arch::X86_64 => "rdx",
            Arch::AArch64 => "x0",
        };

        let map = HashMap::from([
            ("Addition".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_addition_instr()])),
            ("Subtraction".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_subtraction_instr()])),
            ("Multiplication".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_multiplication_instr()])),
            ("Division".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_division_instr()])),
            ("Exponentiation".to_string(),
            operation(exp_reg_lhs, exp_reg_rhs, exp_result_reg, vec![TARGET_ARCH.get_exponentiation_instr()])),
            ("Modulo".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, modulo_result_reg, vec![TARGET_ARCH.get_modulo_instr()]))
            ]
        );
        ArithmeticInstructions{instrs: map}
    }

    // Insert method
    pub fn insert(&mut self, key: String, value: ((String, String), String, Vec<String>)) -> Option<((String, String), String, Vec<String>)> {
        self.instrs.insert(key, value)
    }

    // Get method
    pub fn get(&self, key: &String) -> Option<&((String, String), String, Vec<String>)> {
        self.instrs.get(key)
    }
}