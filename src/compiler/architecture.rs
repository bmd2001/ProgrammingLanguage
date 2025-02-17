#[cfg(target_arch = "x86_64")]
pub const TARGET_ARCH: Arch = Arch::X86_64;

#[cfg(target_arch = "aarch64")]
pub const TARGET_ARCH: Arch = Arch::AArch64;

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
            Arch::X86_64 => "mov rax, 1\n\t{exp_label}:\n\tcmp rcx, 0\n\tje {done_label}\n\timul rax, rdx\n\tdec rcx\n\tjmp {exp_label}\n\t{done_label}:",
            Arch::AArch64 => "mov x0, 1\n\texp_label:\n\tcmp x1, #0\n\tbeq done_label\n\tmul x0, x0, x2\n\tsub x1, x1, #1\n\tb exp_label\n\tdone_label:",
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
        match self {
            Arch::X86_64 => "mov rax, 60\n\tmov rdi, 0\n\tsyscall",
            Arch::AArch64 => "mov x8, 93\n\tmov x0, 0\n\tsvc #0",
        }
    }

    pub fn get_mov_number_instr(&self, value: &str) -> String {
        match self {
            Arch::X86_64 => format!("mov rax, {}", value),
            Arch::AArch64 => format!("mov x0, {}", value),
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
            Arch::X86_64 => format!("mov rax, [rsp + {}]", offset * 8),
            Arch::AArch64 => format!("ldr x0, [sp, #{}]", offset * 8),
        }
    }
}
