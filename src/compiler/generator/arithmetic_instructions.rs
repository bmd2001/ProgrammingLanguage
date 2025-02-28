use std::collections::HashMap;
use crate::compiler::generator::architecture::{Arch, TARGET_ARCH};

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
            operation(arith_reg_lhs, arith_reg_rhs, modulo_result_reg, vec![TARGET_ARCH.get_modulo_instr()])),
            ("And".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_and_instr()])),
            ("Or".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_or_instr()])),
            ("Xor".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_xor_instr()])),
            ("Not".to_string(),
            operation(arith_reg_lhs, arith_reg_rhs, arith_result_reg, vec![TARGET_ARCH.get_not_instr()])),
            ]
        );
        ArithmeticInstructions{instrs: map}
    }

    // Get method
    pub fn get(&self, key: &String) -> Option<&((String, String), String, Vec<String>)> {
        self.instrs.get(key)
    }
}



#[cfg(test)]
mod test_arithmetic_instructions {
    use super::*;

    #[test]
    fn test_init() {
        let obj = ArithmeticInstructions::new();

        // Ensure all expected operations exist in the hashmap
        let expected_keys = [
            "Addition", "Subtraction", "Multiplication", "Division",
            "Exponentiation", "Modulo", "And", "Or", "Xor", "Not"
        ];

        for key in expected_keys.iter() {
            assert!(obj.instrs.contains_key(&key.to_string()), "Missing key: {}", key);
        }
    }
    
    #[test]
    fn test_get_all_operations() {
        let obj = ArithmeticInstructions::new();

        let operations = vec![
            ("Addition", TARGET_ARCH.get_addition_instr()),
            ("Subtraction", TARGET_ARCH.get_subtraction_instr()),
            ("Multiplication", TARGET_ARCH.get_multiplication_instr()),
            ("Division", TARGET_ARCH.get_division_instr()),
            ("Exponentiation", TARGET_ARCH.get_exponentiation_instr()),
            ("Modulo", TARGET_ARCH.get_modulo_instr()),
            ("And", TARGET_ARCH.get_and_instr()),
            ("Or", TARGET_ARCH.get_or_instr()),
            ("Xor", TARGET_ARCH.get_xor_instr()),
            ("Not", TARGET_ARCH.get_not_instr()),
        ];

        for (key, expected_instr) in operations {
            if let Some(((lhs, rhs), result, instructions)) = obj.get(&key.to_string()) {
                let (expected_lhs, expected_rhs, expected_result) = match key {
                    "Exponentiation" => match TARGET_ARCH {
                        Arch::X86_64 => ("rcx", "rdx", "rax"),
                        Arch::AArch64 => ("x1", "x2", "x0"),
                    },
                    "Modulo" => match TARGET_ARCH {
                        Arch::X86_64 => ("rax", "rbx", "rdx"),
                        Arch::AArch64 => ("x0", "x1", "x0"),
                    },
                    _ => match TARGET_ARCH {
                        Arch::X86_64 => ("rax", "rbx", "rax"),
                        Arch::AArch64 => ("x0", "x1", "x0"),
                    },
                };

                assert_eq!(lhs, expected_lhs, "LHS register mismatch for {}", key);
                assert_eq!(rhs, expected_rhs, "RHS register mismatch for {}", key);
                assert_eq!(result, expected_result, "Result register mismatch for {}", key);
                assert_eq!(instructions.len(), 1, "Unexpected instruction count for {}", key);
                assert_eq!(instructions[0], expected_instr, "Instruction mismatch for {}", key);
            } else {
                panic!("Failed to retrieve {} operation from HashMap", key);
            }
        }
    }

    #[test]
    fn test_get_unknown_operation() {
        let obj = ArithmeticInstructions::new();
        let key = "UnknownOperation".to_string();

        assert!(obj.get(&key).is_none(), "Expected None for an unknown operation");
    }
}
