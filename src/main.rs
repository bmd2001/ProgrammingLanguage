mod tokenizer;
mod parser;

use std::fs;
use std::env;
use crate::tokenizer::Tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    if args.len() < 2 {
        eprintln!("Usage: BRS <file.brs>");
        std::process::exit(1);
    }
    
    let file_path = args[1].as_str();
    println!("In file {file_path}");

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
    let mut tokenizer = Tokenizer::new();
    tokenizer.tokenize(&contents);
}
