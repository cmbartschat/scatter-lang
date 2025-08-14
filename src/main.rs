mod analyze;
mod codegen;
mod interpreter;
mod intrinsics;
mod lang;
mod parser;
mod repl;
mod test_analyze;
mod test_e2e;
mod test_interpreter;
mod test_parser;
mod tokenizer;

use clap::Parser;

use crate::{codegen::codegen_module, parser::parse, repl::Repl};

#[derive(Parser, Debug)]
pub struct ReplArgs {
    pub files: Vec<String>,

    /// Analyze code instead of type checking
    #[arg(short, long, default_value_t = false)]
    pub analyze: bool,

    /// Keep REPL open after executing files
    #[arg(short, long, default_value_t = false)]
    pub interactive: bool,

    /// Generate C code for the provided file
    #[arg(short, long, default_value_t = false)]
    pub generate: bool,
}

fn main() {
    let args = ReplArgs::parse();

    if args.generate {
        if args.files.len() != 1 {
            eprintln!("Expected exactly one file provided");
            std::process::exit(1);
        }
        let path = args.files.first().unwrap();
        let source = std::fs::read_to_string(path).expect("Failed to read file");
        let ast = parse(&source).expect("Failed to parse");
        codegen_module(&ast);
        return;
    }

    let repl = Repl::new(args);
    match repl.run() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
