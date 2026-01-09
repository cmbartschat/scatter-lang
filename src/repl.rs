use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Write as _,
    io::{IsTerminal as _, StdoutLock, Write as _, stdin},
    path::{Path, PathBuf},
};

use crate::{
    ReplArgs,
    analyze::{AnalysisError, BlockAnalysisResult, analyze_block_in_namespace, analyze_program},
    codegen::{c::c_codegen_module, js::js_codegen_module, rs::rs_codegen_module},
    interpreter::{BacktraceItem, Interpreter, InterpreterError, InterpreterSnapshot},
    intrinsics::{IntrinsicData, get_intrinsics},
    lang::{ImportLocation, ImportNaming, Module, Term},
    parse_error::ParseError,
    parser::parse,
    path::CanonicalPathBuf,
    program::{FunctionOverwriteStrategy, NamespaceId, NamespaceImport, Program},
};

fn report_arity_inner(result: Option<&BlockAnalysisResult>) -> Cow<'static, str> {
    match result {
        Some(Ok(arity)) => return arity.stringify().into(),
        Some(Err(AnalysisError::IndefiniteSize)) => "unbounded",
        Some(Err(AnalysisError::Pending)) | None => "not resolved",
        Some(Err(AnalysisError::IncompatibleTypes)) => "incompatible types",
        Some(Err(AnalysisError::MissingDeclaration(a))) => {
            return format!("variable {a} is not defined").into();
        }
    }
    .into()
}

fn report_arity(label: &str, result: Option<&BlockAnalysisResult>) {
    #![expect(clippy::print_stdout, reason = "reporting arity")]
    println!("{}: {}", label, report_arity_inner(result));
}

pub struct Repl {
    args: ReplArgs,
    program: Program,
    snapshot: InterpreterSnapshot,
    base_path: PathBuf,
    loaded_paths: HashMap<CanonicalPathBuf, NamespaceId>,
    pending_code: String,
    is_terminal: bool,
}

pub type ReplError = Cow<'static, str>;

type ReplResult<T> = Result<T, ReplError>;

fn stringify_absolute_path(path: Option<&Path>) -> String {
    let Some(path) = path else {
        return "input".into();
    };

    assert!(path.is_absolute(), "Path is not absolute");

    let Ok(cwd) = std::env::current_dir() else {
        return path.display().to_string();
    };

    let Ok(stripped) = Path::strip_prefix(path, cwd) else {
        return path.display().to_string();
    };

    stripped.display().to_string()
}

impl Repl {
    pub fn new(args: ReplArgs, base_path: PathBuf) -> Self {
        Self {
            args,
            snapshot: InterpreterSnapshot::default(),
            program: Program::new(),
            base_path,
            loaded_paths: HashMap::default(),
            pending_code: String::new(),
            is_terminal: stdin().lock().is_terminal(),
        }
    }

    pub fn prepare_code(
        &mut self,
        ast: &Module,
        id: NamespaceId,
        context: &Path,
        function_overwrite_strategy: FunctionOverwriteStrategy,
    ) -> ReplResult<()> {
        let mut imports = vec![];
        for import in &ast.imports {
            match &import.location {
                ImportLocation::Relative(path) => {
                    if !path.starts_with("./") && !path.starts_with("../") {
                        return Err(format!("Invalid import: {}", path).into());
                    }
                    let file_path = CanonicalPathBuf::try_from_path(&context.join(path))
                        .map_err(|e| Cow::Owned(e.to_string()))?;
                    let dependency_id = self.prepare_dependency(&file_path)?;
                    imports.push(NamespaceImport {
                        id: dependency_id,
                        naming: import.naming.clone(),
                    });
                }
            }
        }
        self.program
            .add_functions(id, &ast.functions, function_overwrite_strategy)
            .map_err(|e| format!("Function redefinition error: {}", e))?;

        self.program.add_imports(id, imports);

        Ok(())
    }

    pub fn prepare_dependency(&mut self, path: &CanonicalPathBuf) -> ReplResult<NamespaceId> {
        match self.loaded_paths.get(path) {
            Some(e) => Ok(*e),
            None => self.prepare_file(path).map(|e| e.0),
        }
    }

    pub fn prepare_file(&mut self, path: &CanonicalPathBuf) -> ReplResult<(NamespaceId, Module)> {
        let id = self.program.allocate_namespace();
        self.loaded_paths.insert(path.clone(), id);

        let source = std::fs::read_to_string(path).map_err(|_| "Failed to read file")?;
        let ast = parse(&source)
            .map_err(|e| Self::try_stringify_parse_error(Some(path.as_path()), e, &source))?;
        let Some(context) = path.as_path().parent() else {
            return Err("Unable to resolve file path context".into());
        };

        self.prepare_code(
            &ast,
            id,
            context,
            FunctionOverwriteStrategy::FailOnDuplicate,
        )?;
        self.program.get_namespace_mut(id).path = Some(path.to_owned());
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
            Err(e) => {
                if e.is_early_eof() {
                    std::mem::swap(&mut full_source, &mut self.pending_code);
                    return Ok(());
                }
                return Err(Self::try_stringify_parse_error(None, e, &full_source));
            }
        };
        self.prepare_code(&ast, id, base.as_path(), FunctionOverwriteStrategy::Replace)?;
        self.consume_ast(id, &ast)
    }

    fn consume_ast(&mut self, namespace: NamespaceId, ast: &Module) -> ReplResult<()> {
        if let Some(lang) = &self.args.generate {
            let str = match lang.as_str() {
                "c" => c_codegen_module(&self.program, namespace, &ast.body),
                "js" => js_codegen_module(&self.program, namespace, &ast.body),
                "rs" => rs_codegen_module(&self.program, namespace, &ast.body),
                _ => return Err("Expected 'c', 'js', or 'rs' for generation mode".into()),
            }?;
            {
                #![expect(clippy::print_stdout, reason = "codegen output")]
                println!("{str}");
            }
            return Ok(());
        }
        if self.args.analyze {
            let arities = analyze_program(&self.program);
            for func in &ast.functions {
                report_arity(&func.name, arities[namespace].get(&func.name));
            }
            if !ast.body.terms.is_empty() {
                report_arity(
                    "<body>",
                    Some(&analyze_block_in_namespace(
                        &arities,
                        namespace,
                        &ast.body,
                        &self.program,
                    )),
                );
            }
            Ok(())
        } else {
            let mut snap = InterpreterSnapshot::default();
            std::mem::swap(&mut snap, &mut self.snapshot);
            let interpreter = Interpreter::from_snapshot(snap, &self.program);
            self.snapshot = interpreter
                .execute(namespace, &ast.body)
                .map_err(|e| self.try_stringify_backtrace(e.0, &e.1))?;
            Ok(())
        }
    }

    pub fn load_file(&mut self, path: &str) -> ReplResult<()> {
        let file_path = CanonicalPathBuf::try_from_path(&self.base_path.join(path))
            .map_err(|e| Cow::Owned(e.to_string()))?;
        let (namespace_id, ast) = self.prepare_file(&file_path)?;
        self.consume_ast(namespace_id, &ast)
    }

    pub fn list(&mut self, user_namespace: usize) {
        #![expect(clippy::print_stdout, reason = "listing functions")]

        let analyze = analyze_program(&self.program);

        println!("Available functions:");

        let report_arity = |ns: usize, name: &str| -> Cow<'static, str> {
            report_arity_inner(analyze[ns].get(name))
        };

        let mut column_width: usize = 0;
        column_width = column_width.max(
            self.program.namespaces[user_namespace]
                .functions
                .keys()
                .map(std::string::String::len)
                .max()
                .unwrap_or_default(),
        );
        for import in &self.program.namespaces[user_namespace].imports {
            let max_len = match &import.naming {
                ImportNaming::Wildcard => self.program.namespaces[import.id]
                    .functions
                    .keys()
                    .map(std::string::String::len)
                    .max()
                    .unwrap_or_default(),
                ImportNaming::Named(names) => names
                    .iter()
                    .map(std::string::String::len)
                    .max()
                    .unwrap_or_default(),
                ImportNaming::Scoped(prefix) => {
                    prefix.len()
                        + 1
                        + self.program.namespaces[import.id]
                            .functions
                            .keys()
                            .map(std::string::String::len)
                            .max()
                            .unwrap_or_default()
                }
            };
            column_width = column_width.max(4 + max_len);
        }

        for name in self.program.namespaces[user_namespace].functions.keys() {
            println!(
                "  {name:column_width$}: {}",
                report_arity(user_namespace, name)
            );
        }
        for import in &self.program.namespaces[user_namespace].imports {
            let title = stringify_absolute_path(
                self.program
                    .get_namespace(import.id)
                    .path
                    .as_ref()
                    .map(super::path::CanonicalPathBuf::as_path),
            );

            println!("\n  ╒{:═<20} Imported from: {title}", "",);
            match &import.naming {
                ImportNaming::Wildcard => {
                    let column_width = column_width.saturating_sub(2);
                    for name in self.program.namespaces[import.id].functions.keys() {
                        println!(
                            "  │ {:column_width$}: {}",
                            name,
                            report_arity(import.id, name)
                        );
                    }
                }
                ImportNaming::Named(names) => {
                    let column_width = column_width.saturating_sub(2);
                    for name in names {
                        println!(
                            "  │ {:column_width$}: {}",
                            name,
                            report_arity(import.id, name)
                        );
                    }
                }
                ImportNaming::Scoped(prefix) => {
                    let column_width = column_width.saturating_sub(2 + prefix.len() + 1);

                    for name in self.program.namespaces[import.id].functions.keys() {
                        println!(
                            "  │ {prefix}.{:column_width$}: {}",
                            name,
                            report_arity(import.id, name)
                        );
                    }
                }
            }
        }

        println!("\nREPL commands:");
        println!("  exit");
        println!("  list");
        println!("  list intrinsics");
        println!("  clear");
    }

    pub fn list_intrinsics() {
        #![expect(clippy::print_stdout, reason = "listing functions")]
        println!("Intrinsics:");
        let column_width = 1 + get_intrinsics()
            .iter()
            .map(|f| f.name.len())
            .max()
            .unwrap_or_default();
        for IntrinsicData { name, arity, .. } in get_intrinsics() {
            println!("  {name:column_width$}: {}", arity.stringify());
        }
    }

    fn write_prompt(&self, io: &mut StdoutLock) -> Result<(), std::io::Error> {
        if self.snapshot.stack.is_empty() {
            write!(io, "> ")?;
        } else {
            write!(io, "{:?} > ", self.snapshot.stack)?;
        }

        Ok(())
    }

    fn prompt(&self) -> ReplResult<Option<String>> {
        if self.is_terminal {
            let mut io = std::io::stdout().lock();
            self.write_prompt(&mut io).map_err(|_| "Output error")?;
            io.flush().map_err(|_| "Flush error")?;
            std::mem::drop(io);
        }

        let mut line = String::new();
        let written = std::io::stdin()
            .read_line(&mut line)
            .map_err(|_| "Stdin read error")?;

        if written == 0 {
            Ok(None)
        } else {
            Ok(Some(line))
        }
    }

    pub fn run(mut self) -> ReplResult<()> {
        if !self.args.files.is_empty() {
            for path in &self.args.files.clone() {
                self.load_file(path)?;
            }

            if !self.snapshot.stack.is_empty() {
                {
                    #![expect(clippy::print_stdout, reason = "printing remainder of stack")]
                    println!("{:?}", self.snapshot.stack);
                }
            }
            return Ok(());
        }

        let user_namespace = self.program.allocate_namespace();
        loop {
            let input = self.prompt()?;
            match input {
                None => {
                    if !self.is_terminal && !self.snapshot.stack.is_empty() {
                        {
                            #![expect(clippy::print_stdout, reason = "Output")]
                            println!("{:?}", self.snapshot.stack);
                        }
                    }
                    return Ok(());
                }
                Some(input) => match input.trim() {
                    "exit" => return Ok(()),
                    "list" => self.list(user_namespace),
                    "list intrinsics" => Self::list_intrinsics(),
                    "clear" => self.snapshot.stack.clear(),
                    c => match (self.is_terminal, self.load_code(user_namespace, c)) {
                        (_, Ok(())) => {}
                        (true, Err(e)) => {
                            {
                                #![expect(clippy::print_stderr, reason = "print and stay running")]
                                eprintln!("{e}");
                            }
                        }
                        (false, Err(e)) => return Err(e),
                    },
                },
            }
        }
    }

    fn try_stringify_backtrace(
        &self,
        err: InterpreterError,
        backtrace: &Vec<BacktraceItem>,
    ) -> ReplError {
        match self.stringify_backtrace(err, backtrace) {
            Ok(e) => e,
            Err(e) => e.to_string().into(),
        }
    }

    fn stringify_backtrace(
        &self,
        err: InterpreterError,
        backtrace: &Vec<BacktraceItem>,
    ) -> Result<InterpreterError, std::fmt::Error> {
        let unknown = "Unknown";
        let mut has_name = false;
        let max_name_width = backtrace
            .iter()
            .map(|e| {
                if let Term::Name(n, _) = e.1 {
                    has_name = true;
                    n.len()
                } else if let Term::Capture(n, _) = e.1 {
                    has_name = true;
                    n.len()
                } else {
                    unknown.len()
                }
            })
            .max()
            .unwrap_or_default()
            .min(24)
            + 4;

        if !has_name {
            return Ok(err);
        }

        let mut res = String::with_capacity(1000);
        {
            res.push_str("\n╒═════════════════════════════ Runtime Error\n│\n│  ");
            res.push_str(&err);
            res.push('\n');

            for (i, (namespace, term)) in backtrace.iter().rev().enumerate() {
                let ns = self.program.get_namespace(*namespace);
                let prefix = if i == 0 {
                    "│\n└─ at:  "
                } else {
                    "        "
                };
                res.push_str(prefix);

                write!(
                    res,
                    "{:max_name_width$} {}",
                    if let Term::Name(name, _) = term {
                        name
                    } else if let Term::Capture(name, _) = term {
                        name
                    } else {
                        unknown
                    },
                    stringify_absolute_path(
                        ns.path.as_ref().map(super::path::CanonicalPathBuf::as_path)
                    ),
                )?;

                if let Term::Name(_, loc) = term {
                    write!(res, ":{:?}", loc.start)?;
                }
                if let Term::Capture(_, loc) = term {
                    write!(res, ":{:?}", loc.start)?;
                }

                res.push('\n');
            }
        }

        Ok(res.into())
    }

    fn try_stringify_parse_error(
        path: Option<&Path>,
        err: ParseError,
        source_code: &str,
    ) -> ReplError {
        match Self::stringify_parse_error(path, err, source_code) {
            Ok(e) => e.into(),
            Err(e) => e.to_string().into(),
        }
    }

    fn stringify_parse_error(
        path: Option<&Path>,
        err: ParseError,
        source_code: &str,
    ) -> Result<String, std::fmt::Error> {
        let mut res_owned = String::with_capacity(1000);
        let line_number_width = 6;

        {
            let res = &mut res_owned;

            res.push_str("\n╒═════════════════════════════ Syntax Error\n");

            let (message, loc, info) = err.into_details();

            writeln!(res, "│\n│   {message}\n│")?;

            if let Some(file) = path.map(|e| stringify_absolute_path(Some(e))) {
                writeln!(res, "@ {file}:{:?}\n│", loc.end)?;
            }

            if loc.start.line == loc.end.line {
                if let Some(line) = loc.start.extract(source_code) {
                    writeln!(
                        res,
                        "└─{0:─>line_number_width$}───{0:─>1$}┐",
                        "", loc.end.column
                    )?;
                    writeln!(
                        res,
                        "  {: >line_number_width$} │ {}\n",
                        loc.start.line + 1,
                        line
                    )?;
                }
            } else {
                for (number, line) in loc.extract(source_code) {
                    writeln!(res, "│ {: >line_number_width$} │ {}", number + 1, line)?;
                }
                writeln!(
                    res,
                    "└─{0:─>line_number_width$}───{0:─>1$}┘\n",
                    "", loc.end.column
                )?;
            }

            if let Some(info) = info {
                writeln!(res, "    INFO: {info}")?;
            }
        }

        Ok(res_owned)
    }
}
