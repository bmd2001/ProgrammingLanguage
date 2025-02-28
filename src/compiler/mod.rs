mod tokenizer;
mod generator;
mod logger;
mod parser;
mod span;

use std::sync::{Arc, Mutex};
use self::logger::Logger;
use self::parser::ParserLogger;
use self::tokenizer::{Token, Tokenizer};
use self::parser::{NodeProgram, Parser};
use self::generator::Generator;


pub struct Compiler {
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
        }
    }

    pub fn compile(&mut self, file: &str, input: &str) -> Option<String> {
        // Tokenize
        let tokens: Vec<Token> = {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize(input);
            tokenizer.get_tokens()
        };

        // Parse
        let prog : Option<NodeProgram> = {
            let logger = Arc::new(Mutex::new(ParserLogger::new(file.to_string(), input.to_string())));
            let mut parser = Parser::new(tokens, logger);
            parser.parse()
        };

        // Generate
        let out: String = if let Some(prog) = prog {
            let mut generator = Generator::new(prog);
            generator.generate();
            generator.get_out_assembly()
        } else { return None };
        
        // Return the generated assembly
        Some(out)
    }
}
