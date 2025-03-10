use crate::compiler::generator::subroutines::Subroutines;
use crate::utility::{Arch, OS, TARGET_ARCH, TARGET_OS};

pub const INSTRUCTION_FACTORY: InstructionFactory = InstructionFactory{};

pub struct InstructionFactory{
    
}

impl InstructionFactory {
    // Comments
    pub fn generate_comment(&self, comment: &str) -> String {
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::AArch64, OS::Linux) => format!("\t// {}\n", comment),
            _ => format!("\t; {}\n", comment)
        }
    }
    
    // Arithmetic operations
    pub fn get_addition_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "add rax, rbx",
            Arch::AArch64 => "add x0, x1, x2",
        }
    }

    pub fn get_subtraction_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "sub rax, rbx",
            Arch::AArch64 => "sub x0, x1, x2",
        }
    }

    pub fn get_multiplication_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "mul rbx",
            Arch::AArch64 => "mul x0, x1, x2",
        }
    }

    pub fn get_division_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "xor rdx, rdx\n\tdiv rbx",
            Arch::AArch64 => "sdiv x0, x1, x2",
        }
    }

    pub fn get_modulo_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "xor rdx, rdx\n\tdiv rbx",
            Arch::AArch64 => "sdiv x3, x1, x2\n\tmsub x0, x3, x2, x1",
        }
    }

    pub fn get_exponentiation_instr(&self) -> &str {
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, _) => "mov rax, 1\n{exp_label}:\n\tcmp rcx, 0\n\tje {done_label}\n\timul rax, rdx\n\tdec rcx\n\tjmp {exp_label}\n{done_label}:",
            (Arch::AArch64, OS::Linux) => "mov x0, #1\n{exp_label}:\n\tcmp x1, #0\n\tbeq {done_label}\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb {exp_label}\n{done_label}:",
            (Arch::AArch64, OS::Windows) => "mov x0, #1\n{exp_label}:\n\tcmp x1, #0\n\tbeq {done_label}\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb {exp_label}\n{done_label}:",
            (Arch::AArch64, _) => "mov x0, 1\n{exp_label}:\n\tcmp x1, #0\n\tbeq {done_label}\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb {exp_label}\n{done_label}:"
        }
    }

    pub fn get_mov_number_instr(&self, value: &str) -> String {
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, _) => format!("mov rax, {}", value),
            (Arch::AArch64, OS::Linux) => format!("mov x0, #{}", value),
            (Arch::AArch64, OS::Windows) => format!("mov x0, #{}", value),
            (Arch::AArch64, _) => format!("mov x0, {}", value)
        }
    }

    pub fn get_mov_boolean_instr(&self, value: bool) -> String {
        let bool_as_int = if value {1} else {0};
        match TARGET_ARCH {
            Arch::X86_64 => format!("mov rax, {}", bool_as_int),
            Arch::AArch64 => format!("mov x0, {}", bool_as_int)
        }
    }

    pub fn get_load_variable_instr(&self, offset: usize) -> String {
        match TARGET_ARCH {
            Arch::X86_64 => format!("mov rax, [rsp + {}]", offset),
            Arch::AArch64 => format!("ldr x0, [sp, #{}]", offset),
        }
    }

    // Logical operations
    pub fn get_and_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "and rax, rbx",
            Arch::AArch64 => "and x0, x1, x2",
        }
    }

    pub fn get_or_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "or rax, rbx",
            Arch::AArch64 => "orr x0, x1, x2",
        }
    }

    pub fn get_xor_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "xor rax, rbx",
            Arch::AArch64 => "eor x0, x1, x2",
        }
    }

    pub fn get_not_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "xor rax, 1",
            Arch::AArch64 => "eor x0, x0, #1",
        }
    }

    // System operations
    pub fn get_program_header(&self) -> &str {
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, OS::Windows) => concat!(
                                            "extern ExitProcess\n",
                                            "section .bss\n",
                                            "buffer resb 20\n",
                                            "section .text\n",
                                            "\tglobal _start\n",
                                            "_start:\n",),
            (Arch::AArch64, OS::Windows) => "extern ExitProcess\nglobal _start\n_start:\n",
            (Arch::X86_64, _) => concat!(
                                "section .bss\n",
                                "buffer resb 20\n",
                                "section .text\n",
                                "\tglobal _start\n",
                                "_start:\n",),
            (Arch::AArch64, _) => concat!(
                                ".global _start\n",
                                ".lcomm buffer, 20\n\n",
                                ".text\n",
                                "_start:\n",)
        }
    }

    pub fn get_subroutines(&self) -> String{
        Subroutines::new().generate()
    }

    pub fn get_exit_marker(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "syscall",
            Arch::AArch64 => "svc #0",
        }
    }

    pub fn get_exit_reg(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => "rdi",
            Arch::AArch64 => "x0",
        }
    }

    pub fn get_exit_instr(&self) -> &str {
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, OS::Linux) => "mov rax, 60\n\tmov rdi, 0\n\tsyscall",
            (Arch::X86_64, OS::Windows) => "mov rcx, 0\n\tcall ExitProcess",
            (Arch::X86_64, _) => "mov rax, 0x2000001\n\tmov rdi, 0\n\tsyscall",
            (Arch::AArch64, OS::Linux) => "mov x8, #93\n\tmov x0, #0\n\tsvc #0",
            (Arch::AArch64, OS::Windows) => "mov x0, 0\n\tbl ExitProcess",
            (Arch::AArch64, _) => "ldr x16, =0x2000001\n\tmov x0, 0\n\tsvc #0x80"
        }
    }

    pub fn get_print_instr(&self) -> &str {
        match TARGET_ARCH {
            Arch::X86_64 => {
                concat!(
                "\tlea rdi, [rel buffer+19]\n",
                "\tcall int_to_string\n",
                "\tmov rsi, rdi\n",
                "\tinc rsi\n",
                "\tinc rsi\n",
                "\tcall print_string\n",
                )
            },
            Arch::AArch64 => {
                concat!(
                "\tldr x0, =buffer\n",
                "\tadd x0, x0, 19\n",
                "\tbl int_to_string\n",
                "\tmov x1, x0\n",
                "\tadd x1, x1, 2\n",
                "\tbl print_string\n",
                )
            }
        }
    }

    pub fn get_push_instr(&self, reg: &str) -> String {
        match TARGET_ARCH {
            Arch::X86_64 => {format!("\tpush {}\n", reg)}
            Arch::AArch64 => {format!("\tsub sp, sp, #16\n\tstr {}, [sp, #8]\n", reg)}
        }
    }
    
    pub fn get_pop_instr(&self, reg: &str) -> String {
        match TARGET_ARCH {
            Arch::X86_64 => {format!("\tpop {}\n", reg)}
            Arch::AArch64 => {format!("\tldr {}, [sp, #8]\n\tadd sp, sp, #16\n", reg)}
        }
    }
}



#[cfg(test)]
mod test_architecture{
    use super::*;
    
    #[test]
    fn test_add(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_addition_instr(), "add rax, rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_addition_instr(), "add x0, x1, x2"),
        }
    }

    #[test]
    fn test_sub(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_subtraction_instr(), "sub rax, rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_subtraction_instr(), "sub x0, x1, x2"),
        }
    }

    #[test]
    fn test_mul(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_multiplication_instr(), "mul rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_multiplication_instr(), "mul x0, x1, x2"),
        }
    }

    #[test]
    fn test_div(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_division_instr(), "xor rdx, rdx\n\tdiv rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_division_instr(), "sdiv x0, x1, x2"),
        }
    }
    
    #[test]
    fn test_mod(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_modulo_instr(), "xor rdx, rdx\n\tdiv rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_modulo_instr(), "sdiv x3, x1, x2\n\tmsub x0, x3, x2, x1"),
        }
    }
    
    #[test]
    fn test_exp(){
        let instr_factory = InstructionFactory{};
        let exp_instr = instr_factory.get_exponentiation_instr();
        let expected_instr = match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, _) => concat!(
                                "mov rax, 1\n",
                                "{exp_label}:\n",
                                "\tcmp rcx, 0\n",
                                "\tje {done_label}\n",
                                "\timul rax, rdx\n",
                                "\tdec rcx\n",
                                "\tjmp {exp_label}\n",
                                "{done_label}:"
                                ),
            (Arch::AArch64, OS::Linux) => concat!(
                                "mov x0, #1\n",
                                "{exp_label}:\n",
                                "\tcmp x1, #0\n",
                                "\tbeq {done_label}\n",
                                "\tmul x0, x0, x2\n",
                                "\tsub x1, x1, #1\n",
                                "\tb {exp_label}\n",
                                "{done_label}:"
                                ),
            (Arch::AArch64, _) => concat!(
                                "mov x0, 1\n",
                                "{exp_label}:\n",
                                "\tcmp x1, #0\n",
                                "\tbeq {done_label}\n",
                                "\tmul x0, x0, x2\n",
                                "\tsub x1, x1, #1\n",
                                "\tb {exp_label}\n",
                                "{done_label}:"
                                )
        };
        assert_eq!(exp_instr, expected_instr);
    }
    
    #[test]
    fn test_mov_num(){
        let instr_factory = InstructionFactory{};
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, _) => {
                assert_eq!(instr_factory.get_mov_number_instr("0"), "mov rax, 0");
                assert_eq!(instr_factory.get_mov_number_instr("1"), "mov rax, 1");
            }
            (Arch::AArch64, OS::Linux) => {
                assert_eq!(instr_factory.get_mov_number_instr("0"), "mov x0, #0");
                assert_eq!(instr_factory.get_mov_number_instr("1"), "mov x0, #1");
            }
            (Arch::AArch64, _) => {
                assert_eq!(instr_factory.get_mov_number_instr("0"), "mov x0, 0");
                assert_eq!(instr_factory.get_mov_number_instr("1"), "mov x0, 1");
            }
        }
    }
    
    #[test]
    fn test_mov_bool(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => {
                assert_eq!(instr_factory.get_mov_boolean_instr(true), "mov rax, 1");
                assert_eq!(instr_factory.get_mov_boolean_instr(false), "mov rax, 0");
            },
            Arch::AArch64 => {
                assert_eq!(instr_factory.get_mov_boolean_instr(true), "mov x0, 1");
                assert_eq!(instr_factory.get_mov_boolean_instr(false), "mov x0, 0");
            }
        }
    }
    
    #[test]
    fn test_load(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => {
                assert_eq!(instr_factory.get_load_variable_instr(0), "mov rax, [rsp + 0]");
                assert_eq!(instr_factory.get_load_variable_instr(8), "mov rax, [rsp + 8]");
            },
            Arch::AArch64 => {
                assert_eq!(instr_factory.get_load_variable_instr(0), "ldr x0, [sp, #0]");
                assert_eq!(instr_factory.get_load_variable_instr(8), "ldr x0, [sp, #8]");
            }
        }
    }
    
    #[test]
    fn test_and(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_and_instr(), "and rax, rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_and_instr(), "and x0, x1, x2")
        }
    }
    
    #[test]
    fn test_or(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_or_instr(), "or rax, rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_or_instr(), "orr x0, x1, x2")
        }
    }
    
    #[test]
    fn test_xor(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_xor_instr(), "xor rax, rbx"),
            Arch::AArch64 => assert_eq!(instr_factory.get_xor_instr(), "eor x0, x1, x2")
        }
    }
    
    #[test]
    fn test_not(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_not_instr(), "xor rax, 1"),
            Arch::AArch64 => assert_eq!(instr_factory.get_not_instr(), "eor x0, x0, #1")
        }
    }
    
    #[test]
    fn test_prog_header(){
        let instr_factory = InstructionFactory{};
        match (TARGET_ARCH, TARGET_OS) {
            (Arch::X86_64, OS::Windows) => assert_eq!(instr_factory.get_program_header(),
                                                        concat!(
                                                        "extern ExitProcess\n",
                                                        "section .bss\n",
                                                        "buffer resb 20\n",
                                                        "section .text\n",
                                                        "\tglobal _start\n",
                                                        "_start:\n")),
            (Arch::AArch64, OS::Windows) => assert_eq!(instr_factory.get_program_header(), "extern ExitProcess\nglobal _start\n_start:\n"),
            (Arch::X86_64, _) => assert_eq!(instr_factory.get_program_header(),
                                            concat!(
                                            "section .bss\n",
                                            "buffer resb 20\n",
                                            "section .text\n",
                                            "\tglobal _start\n",
                                            "_start:\n",),),
            (Arch::AArch64, _) => assert_eq!(instr_factory.get_program_header(),
                                            concat!(
                                            ".global _start\n",
                                            ".lcomm buffer, 20\n\n",
                                            ".text\n",
                                            "_start:\n",))
        }
    }
    
    #[test]
    fn test_get_exit_marker(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_exit_marker(),"syscall"),
            Arch::AArch64 => assert_eq!(instr_factory.get_exit_marker(), "svc #0")
        }
    }
    
    #[test]
    fn test_get_exit_reg(){
        let instr_factory = InstructionFactory{};
        match TARGET_ARCH {
            Arch::X86_64 => assert_eq!(instr_factory.get_exit_reg(), "rdi"),
            Arch::AArch64 => assert_eq!(instr_factory.get_exit_reg(), "x0")
        }
    }
    
    #[test]
    fn test_exit(){
        let instr_factory = InstructionFactory{};
        let exit_instr = instr_factory.get_exit_instr();
        let expected_instr = match (TARGET_ARCH, TARGET_OS){
            (Arch::X86_64, OS::Linux) => concat!("mov rax, 60\n",
                                                "\tmov rdi, 0\n",
                                                "\tsyscall"),
            (Arch::X86_64, OS::Windows) => concat!("mov rcx, 0\n",
                                                "\tcall ExitProcess"),
            (Arch::X86_64, _) => concat!("mov rax, 0x2000001\n",
                                        "\tmov rdi, 0\n",
                                        "\tsyscall"),
            (Arch::AArch64, OS::Linux) => concat!("mov x8, #93\n",
                                                "\tmov x0, #0\n",
                                                "\tsvc #0"),
            (Arch::AArch64, OS::Windows) => concat!("mov x0, 0\n",
                                                "\tbl ExitProcess"),
            (Arch::AArch64, _) => concat!("ldr x16, =0x2000001\n",
                                        "\tmov x0, 0\n",
                                        "\tsvc #0x80")
        };
        assert_eq!(exit_instr, expected_instr);
    }
}