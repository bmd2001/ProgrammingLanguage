use crate::utility::{TARGET_ARCH, Arch, TARGET_OS, OS};

pub fn get_print_subroutine() -> String {
    match (TARGET_ARCH, TARGET_OS) {
        (Arch::X86_64, OS::Linux) => get_print_x86_64_linux(),
        (Arch::X86_64, OS::MacOS) => get_print_x86_64_mac(),
        (Arch::X86_64, OS::Windows) => get_print_windows(),
        (Arch::AArch64, OS::Linux) => get_print_aarch64_linux(),
        (Arch::AArch64, OS::MacOS) => get_print_aarch64_mac(),
        _ => get_print_windows()
    }
}

fn get_print_x86_64_mac() -> String {
    concat!(
    "print_string:\n",
    "\tmov rax, 0x2000004\n",
    "\tmov rsi, rdi\n",
    "\tmov rdi, 1\n",
    "\tmov rdx, rcx\n", // length
    "\tsyscall\n",
    "\tmov rax, 0x2000004\n",
    "\tsub rsp, 8\n",
    "\tmov byte [rsp], 0x0A\n",
    "\tmov rsi, rsp\n",
    "\tmov rdx, 1\n",
    "\tsyscall\n",
    "\tadd rsp, 8\n",
    "\tret\n"
    ).to_string()
}

fn get_print_x86_64_linux() -> String {
    concat!(
    "print_string:\n",
    "\tmov rax, 1\n", // syscall: sys_write
    "\tmov rsi, rdi\n", // address of character
    "\tmov rdi, 1\n", // stdout
    "\tmov rdx, rcx\n", // length
    "\tsyscall\n",
    "\tmov rax, 1\n",
    "\tsub rsp, 8\n",
    "\tmov byte [rsp], 0x0A\n",
    "\tmov rsi, rsp\n",
    "\tmov rdx, 1\n",
    "\tsyscall\n",
    "\tadd rsp, 8\n",
    "\tret\n"
    ).to_string()
}

fn get_print_aarch64_linux() -> String {
    concat!(
    "print_string:\n",
    "\tmov x8, #64\n", // syscall: sys_write
    "\tmov x0, #1\n",  // stdout
    "\tsvc #0\n",
    "\tldr x1, =newline\n",
    "\tmov x2, #1\n",
    "\tsvc #0\n",
    "\tret\n"
    ).to_string()
}

fn get_print_aarch64_mac() -> String {
    concat!(
    "print_string:\n",
    "\tldr x16, =0x2000004\n",
    "\tmov x0, #1\n",
    "\tsvc #0x80\n",
    "\tLLD_ADDR x1, newline\n",
    "\tmov x2, #1\n",
    "\tsvc #0x80\n",
    "\tret\n",
    ).to_string()
}

fn get_print_windows() -> String {
    concat!(
        "print_string:\n",
        "\tmov r8, rax\n",       // Save length
        "\tmov rcx, -11\n",       //STD_OUTPUT_HANDLE (-11)
        "\tcall GetStdHandle\n",  // Returns handle in rax
        "\tmov rcx, rax\n",       // Save handle in rcx
        "\tlea rdx, [rdi]\n",
        "\txor r9, r9\n",       
        "\txor r10, r10\n",
        "\tcall WriteFile\n",
        "\tret",
    ).to_string()
}



#[cfg(test)]
mod test_subroutine_print{
    use super::*;
    
    #[test]
    fn test_subroutine(){
        let result = get_print_subroutine();
        match (TARGET_ARCH, TARGET_OS){
            (Arch::X86_64, OS::Linux) => assert_eq!(result, get_print_x86_64_linux()),
            (Arch::X86_64, OS::MacOS) => assert_eq!(result, get_print_x86_64_mac()),
            (Arch::AArch64, OS::Linux) => assert_eq!(result, get_print_aarch64_linux()),
            (Arch::AArch64, OS::MacOS) => assert_eq!(result, get_print_aarch64_mac()),
            _ => assert_eq!(result, get_print_windows())
        }
    }
}