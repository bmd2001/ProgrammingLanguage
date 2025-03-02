#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCH: Arch = Arch::X86_64;

#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCH: Arch = Arch::AArch64;

#[allow(dead_code)] // Only one arch will be used, so it's an expected behaviour
#[derive(Debug)]
pub enum Arch {
    X86_64,
    AArch64,
}

impl Arch {
    // Arithmetic operations
    pub fn get_addition_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "add rax, rbx",
            Arch::AArch64 => "add x0, x1, x2",
        }
    }

    pub fn get_subtraction_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "sub rax, rbx",
            Arch::AArch64 => "sub x0, x1, x2",
        }
    }

    pub fn get_multiplication_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "mul rbx",
            Arch::AArch64 => "mul x0, x1, x2",
        }
    }

    pub fn get_division_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "xor rdx, rdx\n\tdiv rbx",
            Arch::AArch64 => "sdiv x0, x1, x2",
        }
    }

    pub fn get_modulo_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "xor rdx, rdx\n\tdiv rbx",
            Arch::AArch64 => "sdiv x3, x1, x2\n\tmsub x0, x3, x2, x1",
        }
    }

    pub fn get_exponentiation_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "mov rax, 1\n{exp_label}:\n\tcmp rcx, 0\n\tje {done_label}\n\timul rax, rdx\n\tdec rcx\n\tjmp {exp_label}\n{done_label}:",
            Arch::AArch64 => {
                if cfg!(target_os = "linux") {
                    "mov x0, #1\nexp_label:\n\tcmp x1, #0\n\tbeq done_label\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb exp_label\ndone_label:"
                } else {
                    "mov x0, 1\nexp_label:\n\tcmp x1, #0\n\tbeq done_label\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb exp_label\ndone_label:"
                }
            }
        }
    }

    pub fn get_mov_number_instr(&self, value: &str) -> String {
        match self {
            Arch::X86_64 => format!("mov rax, {}", value),
            Arch::AArch64 => {
                if cfg!(target_os = "linux") {
                    format!("mov x0, #{}", value)
                } else {
                    format!("mov x0, {}", value)
                }
            }
        }
    }

    pub fn get_mov_boolean_instr(&self, value: bool) -> String {
        match self {
            Arch::X86_64 => {
                if value { "mov rax, 1".to_string() } else { "mov rax, 0".to_string() }
            },
            Arch::AArch64 => {
                if value { "mov x0, 1".to_string() } else { "mov x0, 0".to_string() }
            },
        }
    }

    pub fn get_load_variable_instr(&self, offset: usize) -> String {
        match self {
            Arch::X86_64 => format!("mov rax, [rsp + {}]", offset),
            Arch::AArch64 => format!("ldr x0, [sp, #{}]", offset),
        }
    }

    // Logical operations
    pub fn get_and_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "and rax, rbx",
            Arch::AArch64 => "and x0, x1, x2",
        }
    }

    pub fn get_or_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "or rax, rbx",
            Arch::AArch64 => "orr x0, x1, x2",
        }
    }

    pub fn get_xor_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "xor rax, rbx",
            Arch::AArch64 => "eor x0, x1, x2",
        }
    }

    pub fn get_not_instr(&self) -> &str {
        match self {
            Arch::X86_64 => "xor rax, 1",
            Arch::AArch64 => "eor x0, x0, #1",
        }
    }

    // System operations
    pub fn get_program_header(&self) -> &str {
        match self {
            Arch::X86_64 => "global _start\n_start:\n",
            Arch::AArch64 => ".global _start\n_start:\n",
        }
    }

    pub fn get_exit_marker(&self) -> &str {
        match self {
            Arch::X86_64 => "syscall",
            Arch::AArch64 => "svc #0",
        }
    }

    pub fn get_exit_reg(&self) -> &str {
        match self {
            Arch::X86_64 => "rdi",
            Arch::AArch64 => "x0",
        }
    }

    pub fn get_exit_instr(&self) -> &str {
        #[cfg(target_os = "linux")]
        {
            match self {
                Arch::X86_64 => "mov rax, 60\n\tmov rdi, 0\n\tsyscall",
                Arch::AArch64 => "mov x8, #93\n\tmov x0, #0\n\tsvc #0",
            }
        }
        #[cfg(target_os = "macos")]
        {
            match self {
                Arch::X86_64 => "mov rax, 0x2000001\n\tmov rdi, 0\n\tsyscall",
                Arch::AArch64 => "ldr x16, =0x2000001\n\tmov x0, 0\n\tsvc #0x80",
            }
        }
    }
}



#[cfg(test)]
mod test_architecture{
    use super::*;
    
    fn get_arch() -> Arch{
        #[cfg(target_arch = "x86_64")]
        return Arch::X86_64;

        #[cfg(target_arch = "aarch64")]
        return Arch::AArch64;
    }
    
    #[test]
    fn test_add(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_addition_instr(), "add rax, rbx"),
            Arch::AArch64 => assert_eq!(arch.get_addition_instr(), "add x0, x1, x2"),
        }
    }

    #[test]
    fn test_sub(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_subtraction_instr(), "sub rax, rbx"),
            Arch::AArch64 => assert_eq!(arch.get_subtraction_instr(), "sub x0, x1, x2"),
        }
    }

    #[test]
    fn test_mul(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_multiplication_instr(), "mul rbx"),
            Arch::AArch64 => assert_eq!(arch.get_multiplication_instr(), "mul x0, x1, x2"),
        }
    }

    #[test]
    fn test_div(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_division_instr(), "xor rdx, rdx\n\tdiv rbx"),
            Arch::AArch64 => assert_eq!(arch.get_division_instr(), "sdiv x0, x1, x2"),
        }
    }
    
    #[test]
    fn test_mod(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_modulo_instr(), "xor rdx, rdx\n\tdiv rbx"),
            Arch::AArch64 => assert_eq!(arch.get_modulo_instr(), "sdiv x3, x1, x2\n\tmsub x0, x3, x2, x1"),
        }
    }
    
    #[test]
    fn test_exp(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_exponentiation_instr(),
                                       concat!(
                                       "mov rax, 1\n",
                                       "{exp_label}:\n",
                                       "\tcmp rcx, 0\n",
                                       "\tje {done_label}\n",
                                       "\timul rax, rdx\n",
                                       "\tdec rcx\n",
                                       "\tjmp {exp_label}\n",
                                       "{done_label}:"
                                       )
            ),

            Arch::AArch64 => {
                if cfg!(target_os = "linux") {
                    assert_eq!(arch.get_exponentiation_instr(),
                               concat!(
                               "mov x0, #1\n",
                               "exp_label:\n",
                               "\tcmp x1, #0\n",
                               "\tbeq done_label\n",
                               "\tmul x0, x0, x2\n",
                               "\tsub x1, x1, #1\n",
                               "\tb exp_label\n",
                               "done_label:"
                               )
                    )
                } else {
                    assert_eq!(arch.get_exponentiation_instr(),
                               concat!(
                               "mov x0, 1\n",
                               "exp_label:\n",
                               "\tcmp x1, #0\n",
                               "\tbeq done_label\n",
                               "\tmul x0, x0, x2\n",
                               "\tsub x1, x1, #1\n",
                               "\tb exp_label\n",
                               "done_label:"
                               )
                    )
                }
            },
        }
    }
    
    #[test]
    fn test_mov_num(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => {
                assert_eq!(arch.get_mov_number_instr("0"), "mov rax, 0");
                assert_eq!(arch.get_mov_number_instr("1"), "mov rax, 1");
            },
            Arch::AArch64 => {
                if cfg!(target_os = "linux") {
                    assert_eq!(arch.get_mov_number_instr("0"), "mov x0, #0");
                    assert_eq!(arch.get_mov_number_instr("1"), "mov x0, #1");
                }
                else {
                    assert_eq!(arch.get_mov_number_instr("0"), "mov x0, 0");
                    assert_eq!(arch.get_mov_number_instr("1"), "mov x0, 1");
                }
            },
        }
    }
    
    #[test]
    fn test_mov_bool(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => {
                assert_eq!(arch.get_mov_boolean_instr(true), "mov rax, 1");
                assert_eq!(arch.get_mov_boolean_instr(false), "mov rax, 0");
            },
            Arch::AArch64 => {
                assert_eq!(arch.get_mov_boolean_instr(true), "mov x0, 1");
                assert_eq!(arch.get_mov_boolean_instr(false), "mov x0, 0");
            }
        }
    }
    
    #[test]
    fn test_load(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => {
                assert_eq!(arch.get_load_variable_instr(0), "mov rax, [rsp + 0]");
                assert_eq!(arch.get_load_variable_instr(8), "mov rax, [rsp + 8]");
            },
            Arch::AArch64 => {
                assert_eq!(arch.get_load_variable_instr(0), "ldr x0, [sp, #0]");
                assert_eq!(arch.get_load_variable_instr(8), "ldr x0, [sp, #8]");
            }
        }
    }
    
    #[test]
    fn test_and(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_and_instr(), "and rax, rbx"),
            Arch::AArch64 => assert_eq!(arch.get_and_instr(), "and x0, x1, x2")
        }
    }
    
    #[test]
    fn test_or(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_or_instr(), "or rax, rbx"),
            Arch::AArch64 => assert_eq!(arch.get_or_instr(), "orr x0, x1, x2")
        }
    }
    
    #[test]
    fn test_xor(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_xor_instr(), "xor rax, rbx"),
            Arch::AArch64 => assert_eq!(arch.get_xor_instr(), "eor x0, x1, x2")
        }
    }
    
    #[test]
    fn test_not(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_not_instr(), "xor rax, 1"),
            Arch::AArch64 => assert_eq!(arch.get_not_instr(), "eor x0, x0, #1")
        }
    }
    
    #[test]
    fn test_prog_header(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_program_header(),"global _start\n_start:\n"),
            Arch::AArch64 => assert_eq!(arch.get_program_header(), ".global _start\n_start:\n")
        }
    }
    
    #[test]
    fn test_get_exit_marker(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_exit_marker(),"syscall"),
            Arch::AArch64 => assert_eq!(arch.get_exit_marker(), "svc #0")
        }
    }
    
    #[test]
    fn test_get_exit_reg(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => assert_eq!(arch.get_exit_reg(), "rdi"),
            Arch::AArch64 => assert_eq!(arch.get_exit_reg(), "x0")
        }
    }
    
    #[test]
    fn test_exit(){
        let arch = get_arch();
        match arch {
            Arch::X86_64 => {
                if cfg!(target_os = "linux"){
                    assert_eq!(arch.get_exit_instr(), 
                               concat!("mov rax, 60\n",
                               "\tmov rdi, 0\n",
                               "\tsyscall")
                    )
                } else {
                    assert_eq!(arch.get_exit_instr(),
                               concat!("mov rax, 0x2000001\n",
                                   "\tmov rdi, 0\n",
                                   "\tsyscall")
                    )
                }
            },
            Arch::AArch64 => if cfg!(target_os = "linux"){
                assert_eq!(arch.get_exit_instr(),
                           concat!("mov x8, #93\n",
                               "\tmov x0, #0\n",
                               "\tsvc #0")
                )
            } else {
                assert_eq!(arch.get_exit_instr(),
                           concat!("ldr x16, =0x2000001\n",
                           "\tmov x0, 0\n",
                           "\tsvc #0x80")
                )
            }
        }
    }
}