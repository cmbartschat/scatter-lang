use crate::{
    analyze::{AnalysisError, BlockAnalysisResult, analyze},
    interpreter::{Interpreter, Stack},
    intrinsics::get_intrinsics,
    parser::parse,
};

mod analyze;
mod interpreter;
mod intrinsics;
mod lang;
mod parser;
mod test_analyze;
mod test_e2e;
mod test_interpreter;
mod test_parser;
mod tokenizer;

fn print_stack(stack: &Stack) {
    println!("{stack:?}");
}

fn report_arity(label: &str, result: Option<&BlockAnalysisResult>) {
    match result {
        Some(Ok(arity)) => println!("{}: {}", label, arity.stringify()),
        Some(Err(AnalysisError::IndefiniteSize)) => {
            println!("{}: unbounded", label)
        }
        Some(Err(AnalysisError::Pending)) => {
            println!("{}: not resolved", label)
        }
        None => println!("{}: not resolved", label),
    }
}

fn accept_source_code(ctx: &mut Interpreter, source: &str, should_check_types: bool) {
    let ast = parse(source).expect("Invalid code.");
    if should_check_types {
        let analysis = analyze(&ast);
        for func in ast.functions.iter() {
            report_arity(&func.name, analysis.arities.get(&func.name));
        }
        report_arity("<body>", Some(&analysis.body_arity));
    } else {
        ctx.load(&ast).expect("Execution error");
    }
}

fn main() {
    let mut args = std::env::args().peekable();
    args.next();
    let mut interactive = args.next_if_eq("-i").is_some();
    let should_check_types = args.next_if_eq("-t").is_some();

    let mut ctx = Interpreter::new();
    for path in args {
        let source = std::fs::read_to_string(path).expect("Failed to read file");
        accept_source_code(&mut ctx, &source, should_check_types);
    }

    print_stack(&ctx.stack);

    while interactive {
        let mut v = String::new();
        std::io::stdin()
            .read_line(&mut v)
            .expect("Stdin read error");

        match v.trim() {
            "exit" => {
                interactive = false;
            }
            "list" => {
                println!("Available functions:");
                for (name, _) in ctx.functions.iter() {
                    println!("  {name}");
                }
                println!("Intrinsics:");
                for (name, _) in get_intrinsics().iter() {
                    println!("  {name}");
                }

                println!("Built-ins:");
                println!("  exit");
                println!("  list");
            }
            _ => {
                accept_source_code(&mut ctx, &v, should_check_types);
                print_stack(&ctx.stack);
            }
        }
    }
}
