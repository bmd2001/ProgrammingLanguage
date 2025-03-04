use crate::compiler::generator::architecture::Arch;
use crate::compiler::generator::TARGET_ARCH;

pub fn get_int_to_str_subroutine() -> String{
    match TARGET_ARCH {
        Arch::X86_64 => {get_int_to_str_x86_64()}
        Arch::AArch64 => {get_int_to_str_aarch64()}
    }
}

fn get_int_to_str_x86_64() -> String{
    concat!(
    "int_to_string:\n",
    "\tmov rbx, 10\n",
    "\tmov rcx, 0\n",
    "\tcall .int_to_string_loop\n\n",
    ".int_to_string_loop:\n",
    "\txor rdx, rdx\n",
    "\tdiv rbx\n",
    "\tadd dl, '0'\n",
    "\tmov [rdi], dl\n",
    "\tdec rdi\n",
    "\tinc rcx\n",
    "\tcmp rax, 0\n",
    "\tjnz .int_to_string_loop\n",
    "\tret\n"
    ).to_string()
}

fn get_int_to_str_aarch64() -> String{
    concat!(
    "int_to_string:\n",
    "\tmov w1, 10\n",
    "\tmov w2, 0\n",
    "\tb .int_to_string_loop\n\n",
    ".int_to_string_loop:\n",
    "\tudiv w3, w0, w1\n",
    "\tmsub w4, w3, w1, w0\n",
    "\tadd w4, w4, '0'\n",
    "\tstrb w4, [x0], -1\n",
    "\tmov w0, w3\n",
    "\tadd w2, w2, 1\n",
    "\tcmp w0, 0\n",
    "\tbne .int_to_string_loop\n",
    "\tret\n"
    ).to_string()
}