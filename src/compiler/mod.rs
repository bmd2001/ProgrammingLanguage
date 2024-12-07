pub mod tokenizer;
pub mod parser;
pub mod generator;

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

    pub fn compile(&mut self, input: &str) -> Result<String, String> {
        // Tokenize
        let tokens: Vec<Token> = {
            let mut tokenizer = Tokenizer::new();
            tokenizer.tokenize(input);
            tokenizer.get_tokens()
        };

        // Parse
        let prog : NodeProgram = {
            let mut parser = Parser::new(tokens);
            parser.parse().map_err(|e| format!("Parse error: {}", e))?
        };

        // Generate
        let out: String = {
            let mut generator = Generator::new(prog);
            generator.generate();
            generator.get_out_assembly()
        };
        
        // Return the generated assembly
        Ok(out)
    }
}
