use std::{
    collections::HashSet,
    io::{StdoutLock, Write},
    ops::Not,
    path::{Path, PathBuf},
};

use crate::{
    ReplArgs,
    analyze::{AnalysisError, BlockAnalysisResult, analyze},
    interpreter::Interpreter,
    intrinsics::get_intrinsics,
    lang::{ImportLocation, Module},
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
    base_path: PathBuf,
    loaded_paths: HashSet<PathBuf>,
}

type ReplResult = Result<(), &'static str>;

impl Repl {
    pub fn new(args: ReplArgs, base_path: PathBuf) -> Self {
        Self {
            args,
            ctx: Interpreter::new(),
            base_path,
            loaded_paths: HashSet::default(),
        }
    }

    pub fn prepare_code(&mut self, source: &str, context: &Path) -> Result<Module, &'static str> {
        let ast = parse(source)?;

        for import in ast.imports.iter() {
            match &import.location {
                ImportLocation::Relative(path) => {
                    if !path.starts_with("./") {
                        eprintln!("Invalid import: {}", path);
                        return Err("Import is not a relative path");
                    }
                    let resolved = context.join(path);
                    self.prepare_dependency(resolved)?;
                }
            }
        }

        self.ctx.load_functions(&ast)?;

        Ok(ast)
    }

    pub fn prepare_dependency(&mut self, path: PathBuf) -> Result<(), &'static str> {
        if self.loaded_paths.contains(&path) {
            return Ok(());
        }

        self.prepare_file(path)?;

        Ok(())
    }

    pub fn prepare_file(&mut self, path: PathBuf) -> Result<Module, &'static str> {
        if !path.is_absolute() {
            return Err("File paths should be absolute");
        }

        self.loaded_paths.insert(path.clone());

        let source = std::fs::read_to_string(&path).map_err(|_| "Failed to read file")?;
        let context = match path.parent() {
            Some(p) => p,
            None => return Err("Unable to resolve file path context"),
        };
        let ast = self.prepare_code(&source, context)?;
        Ok(ast)
    }

    pub fn load_code(&mut self, source: &str) -> ReplResult {
        let base = self.base_path.clone();
        let ast = self.prepare_code(source, base.as_path())?;
        self.consume_ast(ast)
    }

    fn consume_ast(&mut self, ast: Module) -> ReplResult {
        if self.args.analyze {
            let analysis = analyze(&ast, &self.ctx.functions);
            for func in ast.functions.iter() {
                report_arity(&func.name, analysis.arities.get(&func.name));
            }
            if ast.body.terms.is_empty().not() {
                report_arity("<body>", Some(&analysis.body_arity));
            }
            Ok(())
        } else {
            self.ctx.enable_stdin();
            self.ctx.evaluate_block(&ast.body)?;
            self.ctx.disable_stdin();
            Ok(())
        }
    }

    pub fn load_file(&mut self, path: &str) -> ReplResult {
        let ast = self.prepare_file(self.base_path.join(PathBuf::from(path)))?;
        self.consume_ast(ast)
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
