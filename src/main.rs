mod analyze;
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

use crate::repl::Repl;

#[derive(Parser, Debug)]
pub struct ReplArgs {
    pub files: Vec<String>,

    /// Analyze code instead of type checking
    #[arg(short, long, default_value_t = false)]
    pub analyze: bool,

    /// Keep REPL open after executing files
    #[arg(short, long, default_value_t = false)]
    pub interactive: bool,
}

fn main() {
    let args = ReplArgs::parse();
    let repl = Repl::new(args);
    match repl.run() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
