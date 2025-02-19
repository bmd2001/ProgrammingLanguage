mod compiler;

use std::fs;
use std::env;
use std::process::Command;
use crate::compiler::Compiler;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: BRS <file.brs>");
        std::process::exit(1);
    }

    let file = Path::new(&args[1]); // No need for `as_str()`

    let mut out_dir = String::from("./");

    if let Some(out_id) = args.iter().position(|s| s == "--outdir") {
        if let Some(dir) = args.get(out_id + 1) {
            out_dir = dir.clone(); // Use owned string to avoid borrowing issues
        }
    }

    let file_name = file.file_name().unwrap().to_str().unwrap();
    let out_asm_file = format!("{}{}", out_dir, file_name.replace(".brs", ".asm"));
    let out_o_file = format!("{}{}", out_dir, file_name.replace(".brs", ".o"));

    println!("In file {}", file.to_str().unwrap());
    println!("Out file {}", out_asm_file);
    println!("Out file {}", out_o_file);
    
    let contents = fs::read_to_string(file)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
    let assembly: Option<String>;
    {
        let mut compiler = Compiler::new();
        assembly = compiler.compile(file.to_str().unwrap(), contents.as_str());
    }
    match assembly {
        Some(assembly_code) => {
            println!("Assembly:\n{assembly_code}");

            // Write the assembly code to the file
            fs::write(&out_asm_file, assembly_code).expect("Unable to write file");

            let arch = env::consts::ARCH;

            // Run NASM (only if running on x86_64) or as (only if running on arm based macs)
            if arch == "x86_64" {
                let nasm_command = Command::new("nasm")
                    .arg("-f")
                    .arg("macho64")
                    .arg(&out_asm_file)
                    .output()
                    .expect("Failed to execute nasm");

                if !nasm_command.status.success() {
                    eprintln!("Failed to run nasm: {}", String::from_utf8_lossy(&nasm_command.stderr));
                    std::process::exit(1);
                }
            } else if arch == "aarch64" {
                let as_command = Command::new("as")
                    .arg("-arch")
                    .arg("arm64")
                    .arg("-o")
                    .arg(&out_o_file)
                    .arg(&out_asm_file)
                    .output()
                    .expect("Failed to execute ARM assembler (`as`)");
            
                if !as_command.status.success() {
                    eprintln!("Failed to run `as`: {}", String::from_utf8_lossy(&as_command.stderr));
                    std::process::exit(1);
                }
            }

            // Run ld
            let ld_command = Command::new("ld")
                .arg("-arch")
                .arg(if arch == "x86_64" { "x86_64" } else { "arm64" })
                .arg("-macos_version_min")
                .arg("11.0.0")
                .arg("-o")
                .arg(out_dir.to_owned() + "out")
                .arg(&out_o_file)
                .arg("-e")
                .arg("_start")
                .arg("-static")
                .output()
                .expect("Failed to execute ld");

            if !ld_command.status.success() {
                eprintln!("Failed to run ld: {}", String::from_utf8_lossy(&ld_command.stderr));
                std::process::exit(1);
            }

            println!("{}", String::from_utf8_lossy(&ld_command.stdout));
        }
        None => {
            std::process::exit(1);
        }
    }
}
