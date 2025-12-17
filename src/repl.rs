use std::{
    borrow::Cow,
    collections::HashMap,
    io::{StdoutLock, Write},
    ops::Not,
    path::{Path, PathBuf},
};

use crate::{
    ReplArgs,
    analyze::{AnalysisError, BlockAnalysisResult, analyze_block_in_namespace, analyze_program},
    codegen::{c::c_codegen_module, js::js_codegen_module, rs::rs_codegen_module},
    interpreter::{Interpreter, InterpreterSnapshot},
    intrinsics::{IntrinsicData, get_intrinsics},
    lang::{ImportLocation, ImportNaming, Module},
    parser::{ParseError, parse},
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
    pending_code: String,
}

pub type ReplError = Cow<'static, str>;

type ReplResult<T> = Result<T, ReplError>;

fn stringify_absolute_path(path: Option<&Path>) -> String {
    let Some(path) = path else {
        return "input".into();
    };

    let Ok(cwd) = std::env::current_dir() else {
        return path.display().to_string();
    };

    let Ok(stripped) = Path::strip_prefix(path, cwd) else {
        return path.display().to_string();
    };

    stripped.display().to_string()
}

fn parse_error_to_cow(path: Option<&Path>, value: ParseError) -> Cow<'static, str> {
    let file = stringify_absolute_path(path);
    match value {
        ParseError::UnboundedString(loc) => {
            Cow::<str>::from(format!("{file}:{:?}: Unclosed string literal", loc))
        }
        ParseError::Location(e, loc) => Cow::<str>::from(format!("{file}:{:?}: {}", loc, e)),
        ParseError::Range(e, loc) => Cow::<str>::from(format!("{file}:{:?}: {}", loc, e)),
        ParseError::UnclosedExpression(e, loc) => {
            Cow::<str>::from(format!("{file}:{:?}: {}", loc, e))
        }
        ParseError::UnexpectedEnd(e) => Cow::<str>::from(format!("{file}: {}", e)),
    }
}

impl Repl {
    pub fn new(args: ReplArgs, base_path: PathBuf) -> Self {
        Self {
            args,
            snapshot: InterpreterSnapshot::default(),
            program: Program::new(),
            base_path,
            loaded_paths: HashMap::default(),
            pending_code: "".into(),
        }
    }

    pub fn prepare_code(
        &mut self,
        ast: &Module,
        id: NamespaceId,
        context: &Path,
    ) -> ReplResult<()> {
        let mut imports = vec![];
        for import in ast.imports.iter() {
            match &import.location {
                ImportLocation::Relative(path) => {
                    if !path.starts_with("./") && !path.starts_with("../") {
                        return Err(format!("Invalid import: {}", path).into());
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

        Ok(())
    }

    pub fn prepare_dependency(&mut self, path: PathBuf) -> ReplResult<NamespaceId> {
        match self.loaded_paths.get(&path) {
            Some(e) => Ok(*e),
            None => self.prepare_file(path).map(|e| e.0),
        }
    }

    pub fn prepare_file(&mut self, path: PathBuf) -> ReplResult<(NamespaceId, Module)> {
        if !path.is_absolute() {
            return Err("File paths should be absolute".into());
        }

        let id = self.program.allocate_namespace();
        self.loaded_paths.insert(path.clone(), id);

        let source = std::fs::read_to_string(&path).map_err(|_| "Failed to read file")?;
        let ast = parse(&source).map_err(|e| parse_error_to_cow(Some(&path), e))?;
        let context = match path.parent() {
            Some(p) => p,
            None => return Err("Unable to resolve file path context".into()),
        };

        self.prepare_code(&ast, id, context)?;
        Ok((id, ast))
    }

    pub fn load_code(&mut self, id: NamespaceId, source: &str) -> ReplResult<()> {
        if !self.pending_code.is_empty() {
            self.pending_code.push('\n');
        }
        self.pending_code.push_str(source);
        let base = self.base_path.clone();
        let mut full_source = String::new();
        std::mem::swap(&mut full_source, &mut self.pending_code);
        let ast = match parse(&full_source) {
            Ok(e) => e,
            Err(e) => match e {
                ParseError::UnexpectedEnd(_)
                | ParseError::UnclosedExpression(..)
                | ParseError::UnboundedString(_) => {
                    std::mem::swap(&mut full_source, &mut self.pending_code);
                    return Ok(());
                }
                e @ ParseError::Location(..) => return Err(parse_error_to_cow(None, e)),
                e @ ParseError::Range(..) => return Err(parse_error_to_cow(None, e)),
            },
        };
        self.prepare_code(&ast, id, base.as_path())?;
        self.consume_ast(id, ast)
    }

    fn consume_ast(&mut self, namespace: NamespaceId, ast: Module) -> ReplResult<()> {
        if let Some(lang) = &self.args.generate {
            match lang.as_str() {
                "c" => c_codegen_module(&self.program, namespace, &ast.body),
                "js" => js_codegen_module(&self.program, namespace, &ast.body),
                "rs" => rs_codegen_module(&self.program, namespace, &ast.body),
                _ => {
                    eprintln!("Expected c or js for generation mode");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
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

    pub fn load_file(&mut self, path: &str) -> ReplResult<()> {
        let (namespace_id, ast) = self.prepare_file(self.base_path.join(PathBuf::from(path)))?;
        self.consume_ast(namespace_id, ast)
    }

    pub fn list(&mut self, user_namespace: usize) -> ReplResult<()> {
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
        for IntrinsicData { name, .. } in get_intrinsics().iter() {
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

    pub fn run(mut self) -> ReplResult<()> {
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
            match self.prompt()?.trim() {
                "exit" => return Ok(()),
                "list" => self.list(user_namespace)?,
                "clear" => self.snapshot.stack.clear(),
                c => self.load_code(user_namespace, c)?,
            };
        }
    }
}
