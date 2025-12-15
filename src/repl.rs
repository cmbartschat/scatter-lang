use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
    ops::Not,
    path::{Path, PathBuf},
};

use crate::{
    ReplArgs,
    analyze::{AnalysisError, BlockAnalysisResult, analyze_block_in_namespace, analyze_program},
    interpreter::{Interpreter, InterpreterSnapshot},
    intrinsics::get_intrinsics,
    lang::{ImportLocation, ImportNaming, Module},
    parser::parse,
    program::{NamespaceId, NamespaceImport, Program},
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
    program: Program,
    snapshot: InterpreterSnapshot,
    base_path: PathBuf,
    loaded_paths: HashMap<PathBuf, NamespaceId>,
}

type ReplResult = Result<(), &'static str>;

impl Repl {
    pub fn new(args: ReplArgs, base_path: PathBuf) -> Self {
        Self {
            args,
            snapshot: InterpreterSnapshot::default(),
            program: Program::new(),
            base_path,
            loaded_paths: HashMap::default(),
        }
    }

    pub fn prepare_code(
        &mut self,
        source: &str,
        id: NamespaceId,
        context: &Path,
    ) -> Result<Module, &'static str> {
        let ast = parse(source)?;

        let mut imports = vec![];
        for import in ast.imports.iter() {
            match &import.location {
                ImportLocation::Relative(path) => {
                    if !path.starts_with("./") {
                        eprintln!("Invalid import: {}", path);
                        return Err("Import is not a relative path");
                    }
                    let resolved = context.join(path);
                    let dependency_id = self.prepare_dependency(resolved)?;
                    imports.push(NamespaceImport {
                        id: dependency_id,
                        naming: import.naming.clone(),
                    });
                }
            }
        }
        self.program.add_functions(id, &ast.functions);
        self.program.add_imports(id, imports);

        Ok(ast)
    }

    pub fn prepare_dependency(&mut self, path: PathBuf) -> Result<NamespaceId, &'static str> {
        match self.loaded_paths.get(&path) {
            Some(e) => Ok(*e),
            None => self.prepare_file(path).map(|e| e.0),
        }
    }

    pub fn prepare_file(&mut self, path: PathBuf) -> Result<(NamespaceId, Module), &'static str> {
        if !path.is_absolute() {
            return Err("File paths should be absolute");
        }

        let id = self.program.allocate_namespace();
        self.loaded_paths.insert(path.clone(), id);

        let source = std::fs::read_to_string(&path).map_err(|_| "Failed to read file")?;
        let context = match path.parent() {
            Some(p) => p,
            None => return Err("Unable to resolve file path context"),
        };
        Ok((id, self.prepare_code(&source, id, context)?))
    }

    pub fn load_code(&mut self, id: NamespaceId, source: &str) -> ReplResult {
        let base = self.base_path.clone();
        let ast = self.prepare_code(source, id, base.as_path())?;
        self.consume_ast(id, ast)
    }

    fn consume_ast(&mut self, namespace: NamespaceId, ast: Module) -> ReplResult {
        if self.args.analyze {
            let arities = analyze_program(&self.program);
            for func in ast.functions.iter() {
                report_arity(&func.name, arities[namespace].get(&func.name));
            }
            if ast.body.terms.is_empty().not() {
                report_arity(
                    "<body>",
                    Some(&analyze_block_in_namespace(
                        &arities,
                        namespace,
                        &ast.body,
                        &self.program,
                    )),
                )
            }
            Ok(())
        } else {
            let mut snap = InterpreterSnapshot::default();
            std::mem::swap(&mut snap, &mut self.snapshot);
            let mut interpreter = Interpreter::from_snapshot(snap, &self.program);
            interpreter.enable_stdin();
            self.snapshot = interpreter.execute(&ast.body)?;
            Ok(())
        }
    }

    pub fn load_file(&mut self, path: &str) -> ReplResult {
        let (namespace_id, ast) = self.prepare_file(self.base_path.join(PathBuf::from(path)))?;
        self.consume_ast(namespace_id, ast)
    }

    pub fn list(&mut self, user_namespace: usize) -> ReplResult {
        println!("Available functions:");
        for (name, _) in self.program.namespaces[user_namespace].functions.iter() {
            println!("  {name}");
        }
        for import in self.program.namespaces[user_namespace].imports.iter() {
            match &import.naming {
                ImportNaming::Wildcard => {
                    for (name, _) in self.program.namespaces[import.id].functions.iter() {
                        println!("  {}", name);
                    }
                }
                ImportNaming::Named(names) => {
                    for name in names.iter() {
                        println!("  {}", name);
                    }
                }
                ImportNaming::Scoped(prefix) => {
                    for (name, _) in self.program.namespaces[import.id].functions.iter() {
                        println!("  {prefix}.{}", name);
                    }
                }
            }
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
        if !self.snapshot.stack.is_empty() {
            write!(io, "{:?} > ", self.snapshot.stack)?;
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
        if !self.args.files.is_empty() {
            for path in &self.args.files.clone() {
                self.load_file(path)?;
            }

            if !self.snapshot.stack.is_empty() {
                println!("{:?}", self.snapshot.stack);
            }
            return Ok(());
        }

        let user_namespace = self.program.allocate_namespace();
        loop {
            let command = self.prompt()?;

            match command.trim() {
                "exit" => return Ok(()),
                "list" => self.list(user_namespace)?,
                "clear" => self.snapshot.stack.clear(),
                _ => self.load_code(user_namespace, &command)?,
            };
        }
    }
}
