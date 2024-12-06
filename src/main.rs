mod tokenizer;
mod parser;
mod generator;

use std::fs;
use std::env;
use std::process::Command;
use crate::generator::Generator;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() < 2 {
        eprintln!("Usage: BRS <file.brs>");
        std::process::exit(1);
    }
    
    let file_path = args[1].as_str();
    let mut out_file : &String = &file_path.replace(".brs", ".asm");

    if args.contains(&"-o".to_string()){
        let out_id = args.iter().position(|s| s == "-o").unwrap()+1;
        if let Some(filename) = args.get(out_id) {
            if filename.contains(".asm"){
                out_file = filename;
            }
        }
    }
    println!("In file {file_path}");
    println!("Out file {out_file}");

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
    let mut assembly= String::new();
    {
        let mut tokenizer = Tokenizer::new();
        tokenizer.tokenize(&contents);
        let tokens: Vec<Token> = tokenizer.get_tokens();
        let mut parser = Parser::new(tokens);
        let prog = parser.parse().unwrap();
        let mut generator = Generator::new(prog);
        generator.generate();
        assembly = generator.get_out_assembly();
    }
    println!("Assembly:\n{assembly}");
    dbg!(out_file);
    fs::write(out_file, assembly).expect("Unable to write file");
    use std::process::Command;

    // Run nasm
    let nasm_command = Command::new("nasm")
        .arg("-f")
        .arg("macho64")
        .arg(out_file)
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
        .arg("out")
        .arg(out_file.replace(".asm", ".o"))
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
