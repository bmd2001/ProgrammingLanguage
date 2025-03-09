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

impl Arch{
    pub fn get_arithmetic_regs(&self) -> (&str, &str, &str){
        match self {
            Arch::X86_64 => ("rax", "rbx", "rax"),
            Arch::AArch64 => ("x0", "x1", "x0"),
        }
    }
    
    pub fn get_exponentiation_regs(&self) -> (&str, &str, &str){
        match self{
            Arch::X86_64 => ("rcx", "rdx", "rax"),
            Arch::AArch64 => ("x1", "x2", "x0"),
        }
    }
    
    pub fn get_modulo_reg(&self) -> &str {
        match self {
            Arch::X86_64 => "rdx",
            Arch::AArch64 => "x0",
        }
    }
}