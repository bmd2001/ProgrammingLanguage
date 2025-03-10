use crate::utility::{TARGET_ARCH, Arch, TARGET_OS, OS};

pub fn get_print_subroutine() -> String {
    match TARGET_ARCH {
        Arch::X86_64 => {
            match TARGET_OS {
                OS::Linux => get_print_x86_64_linux(),
                _ => get_print_x86_64_mac(),
            }
        }
        Arch::AArch64 => {
            match TARGET_OS {
                OS::Linux => get_print_aarch64_linux(),
                _ => get_print_aarch64_mac(),
            }
        }
    }
}

fn get_print_x86_64_mac() -> String {
    concat!(
    "print_string:\n",
    "\tlea rbx, [rel buffer+20]\n",
    "\tmov al, [rsi]\n",
    "\tcmp rsi, rbx\n",
    "\tje .done\n",
    "\tmov rax, 0x2000004\n",
    "\tmov rdi, 1\n",
    "\tmov rdx, 1\n",
    "\tsyscall\n",
    "\tinc rsi\n",
    "\tjmp print_string\n",
    ".done:\n",
    "\tpush 10\n",
    "\tlea rsi, [rsp]\n",
    "\tmov rax, 0x2000004\n",
    "\tmov rdi, 1\n",
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
    "\tmov rdi, 1\n", // stdout
    "\tmov rdx, 1\n", // write 1 byte
    ".loop:\n",
    "\tmov rsi, rbx\n", // address of character
    "\tsyscall\n",
    "\tinc rbx\n",
    "\tcmp byte [rbx], 0\n",
    "\tjne .loop\n",
    "\tret\n"
    ).to_string()
}

fn get_print_aarch64_linux() -> String {
    concat!(
    "print_string:\n",
    "\tmov x8, 64\n", // syscall: sys_write
    "\tmov x0, 1\n",  // stdout
    "\tmov x2, 1\n",  // write 1 byte
    ".loop:\n",
    "\tsvc #0\n",
    "\tadd x1, x1, 1\n",
    "\tldrb w3, [x1]\n",
    "\tcbnz w3, .loop\n",
    "\tret\n"
    ).to_string()
}

fn get_print_aarch64_mac() -> String {
    concat!(
    "print_string:\n",
    "\tldr x16, =0x2000004\n", // macOS syscall number for write
    "\tmov x0, 1\n",  // stdout
    "\tmov x2, 1\n",  // write 1 byte
    ".loop:\n",
    "\tsvc #0x80\n",
    "\tadd x1, x1, 1\n",
    "\tldrb w3, [x1]\n",
    "\tcbnz w3, .loop\n",
    "\tret\n"
    ).to_string()
}



#[cfg(test)]
mod test_subroutine_print{
    use super::*;
    
    #[test]
    fn test_subroutine(){
        let result = get_print_subroutine();
        match TARGET_ARCH {
            Arch::X86_64 => {
                match TARGET_OS {
                    OS::Linux => assert_eq!(result, get_print_x86_64_linux()),
                    _ => assert_eq!(result, get_print_x86_64_mac()),
                }
            }
            Arch::AArch64 => {
                match TARGET_OS {
                    OS::Linux => assert_eq!(result, get_print_aarch64_linux()),
                    _ => assert_eq!(result, get_print_aarch64_mac())
                }
            }
        }
    }
}