pub mod tokenizer;
pub mod parser;
pub mod generator;
mod architecture;
mod arithmetic_instructions;
pub mod logger;
mod stack_handler;

use crate::compiler::tokenizer::{Token, Tokenizer};
use crate::compiler::parser::{NodeProgram, Parser};
use crate::compiler::generator::Generator;

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
            let mut parser = Parser::new(tokens, file.to_string(), input.to_string());
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
