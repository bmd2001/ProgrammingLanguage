use std::collections::HashMap;

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
        let map = HashMap::from([
            ("Addition".to_string(),
            operation("rax","rbx", "rax", vec!["add rax, rbx"])),
            ("Subtraction".to_string(),
            operation("rax","rbx", "rax", vec!["sub rax, rbx"])),
            ("Multiplication".to_string(),
            operation("rax","rbx", "rax", vec!["mul rbx"])),
            ("Division".to_string(),
            operation("rax","rbx", "rax", vec!["xor rdx, rdx", "div rbx"])),
            ("Exponentiation".to_string(),
            operation("rax","rbx", "rax", vec![
            "mov rax, 1", "{exp_label}:", "cmp rcx, 0", "je {done_label}", "imul rax, rdx", "dec rcx", "jmp {exp_label}", "{done_label}:"
            ])),
            ("Modulo".to_string(),
            operation("rax","rbx", "rdx", vec!["xor rdx, rdx", "div rbx"]))
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