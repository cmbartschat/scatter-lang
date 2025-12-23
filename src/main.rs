mod analyze;
mod codegen;
mod convert;
mod interpreter;
mod intrinsics;
mod lang;
mod parser;
mod program;
mod repl;
mod test_analyze;
mod test_convert;
mod test_e2e;
mod test_interpreter;
mod test_parser;
mod tokenizer;
use clap::Parser;

use crate::repl::{Repl, ReplError};

#[derive(Parser, Debug)]
pub struct ReplArgs {
    pub files: Vec<String>,

    /// Analyze code instead of type checking
    #[arg(short, long, default_value_t = false)]
    pub analyze: bool,

    /// Generate code for the provided file
    #[arg(short, long)]
    pub generate: Option<String>,
}

fn main() -> Result<(), ReplError> {
    let args = ReplArgs::parse();

    if args.generate.is_some() && args.files.len() != 1 {
        return Err("Expected exactly one file provided".into());
    }

    let repl = Repl::new(
        args,
        std::env::current_dir().expect("Could not get current directory"),
    );
    repl.run()
}
