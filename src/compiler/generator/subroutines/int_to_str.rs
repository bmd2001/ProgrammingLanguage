use crate::utility::{TARGET_ARCH, Arch, TARGET_OS, OS};

pub fn get_int_to_str_subroutine() -> String{
    match (TARGET_ARCH, TARGET_OS){
        (Arch::X86_64, OS::Windows) => {get_int_to_str_x86_64_win()},
        (Arch::X86_64, _) => {get_int_to_str_x86_64()},
        (Arch::AArch64, _) => {get_int_to_str_aarch64()}
    }
}

fn get_int_to_str_x86_64() -> String{
    concat!(
    "int_to_string:\n",
    "\tmov rbx, 10\n",
    "\tmov rcx, 1\n",
    "\tcall .int_to_string_loop\n",
    "\tret\n\n",
    ".int_to_string_loop:\n",
    "\txor rdx, rdx\n",
    "\tdiv rbx\n",
    "\tadd dl, '0'\n",
    "\tmov [rdi], dl\n",
    "\tdec rdi\n",
    "\tinc rcx\n",
    "\tcmp rax, 0\n",
    "\tjnz .int_to_string_loop\n",
    "\tinc rdi\n",
    "\tmov rax, rcx\n",
    "\tret\n"
    ).to_string()
}

fn get_int_to_str_x86_64_win() -> String{
    concat!(
    "int_to_string:\n",
    "\tmov rbx, 10\n",
    "\tmov rcx, 0\n",
    ".int_to_string_loop:\n",
    "\txor rdx, rdx\n",
    "\tdiv rbx\n",
    "\tadd dl, '0'\n",
    "\tmov [rdi], dl\n",
    "\tdec rdi\n",
    "\tinc rcx\n",
    "\tcmp rax, 0\n",
    "\tjnz .int_to_string_loop\n",
    "\tinc rdi\n",
    "\tmov rax, rcx\n",
    "\tret\n"
    ).to_string()
}

fn get_int_to_str_aarch64() -> String{
    concat!(
    "int_to_string:\n",
    "\tmov x1, 10\n",
    "\tmov x2, 1\n\n",
    ".int_to_string_loop:\n",
    "\tudiv x3, x0, x1\n",
    "\tmsub x4, x3, x1, x0\n",
    "\tadd x4, x4, '0'\n",
    "\tstrb w4, [x7], -1\n",
    "\tmov x0, x3\n",
    "\tadd x2, x2, 1\n",
    "\tcmp x0, 0\n",
    "\tbne .int_to_string_loop\n",
    "\tret\n"
    ).to_string()
}

#[cfg(test)]
mod test_subroutine_int_to_str {
    use super::*;
    
    #[test]
    fn test_subroutine(){
        match (TARGET_ARCH, TARGET_OS) {  
            (Arch::X86_64, OS::Windows) => assert_eq!(get_int_to_str_subroutine(), get_int_to_str_x86_64_win()),
            (Arch::X86_64, _) => assert_eq!(get_int_to_str_subroutine(), get_int_to_str_x86_64()),
            (Arch::AArch64, _) => assert_eq!(get_int_to_str_subroutine(), get_int_to_str_aarch64())
        }
    }
}