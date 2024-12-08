mod compiler;

use std::fs;
use std::env;
use std::process::Command;
use crate::compiler::Compiler;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() < 2 {
        eprintln!("Usage: BRS <file.brs>");
        std::process::exit(1);
    }
    
    let file_path = args[1].as_str();
    let mut out_dir : &String = &String::from("./");

    if args.contains(&"--outdir".to_string()){
        let out_id = args.iter().position(|s| s == "--outdir").unwrap()+1;
        if let Some(dir) = args.get(out_id) {
            out_dir = dir;
        }
    }
    let out_asm_file: &String = &(out_dir.to_owned() + &*file_path.replace(".brs", ".asm"));
    let out_o_file : &String = &(out_dir.to_owned() + &*file_path.replace(".brs", ".o"));
    println!("In file {file_path}");
    println!("Out file {out_asm_file}");
    println!("Out file {out_o_file}");

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
    let assembly: Result<String, String>;
    {
        let mut compiler = Compiler::new();
        assembly = compiler.compile(contents.as_str());
    }
    match assembly {
        Ok(assembly_code) => {
            println!("Assembly:\n{assembly_code}");

            // Write the assembly code to the file
            fs::write(&out_asm_file, assembly_code).expect("Unable to write file");

            // Run nasm
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

            // Run ld
            let ld_command = Command::new("ld")
                .arg("-arch")
                .arg("x86_64")
                .arg("-macos_version_min")
                .arg("10.9.0")
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
        Err(e) => {
            eprintln!("Error during compilation: {}", e);
            std::process::exit(1);
        }
    }
}
