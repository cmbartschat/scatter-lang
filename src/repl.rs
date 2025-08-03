use std::io::{StdoutLock, Write};

use crate::{
    ReplArgs,
    analyze::{AnalysisError, BlockAnalysisResult, analyze},
    interpreter::Interpreter,
    intrinsics::get_intrinsics,
    parser::parse,
};

fn report_arity(label: &str, result: Option<&BlockAnalysisResult>) {
    match result {
        Some(Ok(arity)) => println!("{}: {}", label, arity.stringify()),
        Some(Err(AnalysisError::IndefiniteSize)) => {
            println!("{}: unbounded", label)
        }
        Some(Err(AnalysisError::Pending)) => {
            println!("{}: not resolved", label)
        }
        Some(Err(AnalysisError::IncompatibleTypes)) => {
            println!("{}: incompatible types", label)
        }
        None => println!("{}: not resolved", label),
    }
}

pub struct Repl {
    args: ReplArgs,
    ctx: Interpreter,
}

type ReplResult = Result<(), &'static str>;

impl Repl {
    pub fn new(args: ReplArgs) -> Self {
        Self {
            args,
            ctx: Interpreter::new(),
        }
    }

    pub fn load_code(&mut self, source: &str) -> ReplResult {
        let ast = parse(source).expect("Invalid code.");
        if self.args.analyze {
            let analysis = analyze(&ast);
            for func in ast.functions.iter() {
                report_arity(&func.name, analysis.arities.get(&func.name));
            }
            report_arity("<body>", Some(&analysis.body_arity));
            Ok(())
        } else {
            self.ctx.load(&ast)
        }
    }

    pub fn load_file(&mut self, path: &str) -> ReplResult {
        let source = std::fs::read_to_string(path).map_err(|_| "Failed to read file")?;
        self.load_code(&source)
    }

    pub fn list(&mut self) -> ReplResult {
        println!("Available functions:");
        for (name, _) in self.ctx.functions.iter() {
            println!("  {name}");
        }
        println!("Intrinsics:");
        for (name, _) in get_intrinsics().iter() {
            println!("  {name}");
        }

        println!("REPL commands:");
        println!("  exit");
        println!("  list");
        println!("  clear");

        Ok(())
    }

    fn write_prompt(&self, io: &mut StdoutLock) -> Result<(), std::io::Error> {
        if !self.ctx.stack.is_empty() {
            write!(io, "{:?} > ", self.ctx.stack)?;
        } else {
            write!(io, "> ")?;
        }

        Ok(())
    }

    fn prompt(&self) -> Result<String, &'static str> {
        let mut io = std::io::stdout().lock();
        self.write_prompt(&mut io).map_err(|_| "Output error")?;
        io.flush().map_err(|_| "Flush error")?;
        std::mem::drop(io);

        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(|_| "Stdin read error")?;

        Ok(line)
    }

    pub fn run(mut self) -> ReplResult {
        let interactive = self.args.interactive || self.args.files.is_empty();

        for path in &self.args.files.clone() {
            self.load_file(path)?;
        }

        if !interactive {
            if !self.ctx.stack.is_empty() {
                println!("{:?}", self.ctx.stack);
            }
            return Ok(());
        }

        loop {
            let command = self.prompt()?;

            match command.trim() {
                "exit" => return Ok(()),
                "list" => self.list()?,
                "clear" => self.ctx.stack.clear(),
                _ => self.load_code(&command)?,
            };
        }
    }
}
