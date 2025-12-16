use crate::{
    codegen::context::CodegenContext,
    lang::{Block, Loop, Term},
    program::{NamespaceId, Program},
};

static DEFS: &str = include_str!("./rs-defs.rs");
static INTRINSICS: &str = include_str!("../intrinsics.rs");
static INTERPRETER: &str = include_str!("../interpreter.rs");

fn codegen_loop_condition(ctx: &mut CodegenContext, block: &Option<Block>) {
    if let Some(e) = block {
        codegen_block(ctx, e);
        ctx.target.write_line("if !c.check_condition()? { break }");
    }
}

fn codegen_loop(ctx: &mut CodegenContext, loop_t: &Loop) {
    ctx.target.write_line("loop {");
    ctx.target.increase_indent();
    codegen_loop_condition(ctx, &loop_t.pre_condition);
    codegen_block(ctx, &loop_t.body);
    codegen_loop_condition(ctx, &loop_t.post_condition);
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
}

fn codegen_term(ctx: &mut CodegenContext, term: &Term) {
    match term {
        Term::String(e) => ctx.target.write_line(&format!("c.push({:?})?;", e)),
        Term::Number(e) => ctx.target.write_line(&format!("c.push({}f64)?;", e)),
        Term::Bool(true) => ctx.target.write_line("c.push(true)?;"),
        Term::Bool(false) => ctx.target.write_line("c.push(false)?;"),
        Term::Address(a) => ctx.target.write_line(&format!(
            "c.push(&({} as Operation))?;",
            ctx.resolve_name_reference(a)
        )),
        Term::Name(n) => ctx
            .target
            .write_line(&format!("{}(c)?;", ctx.resolve_name_reference(n))),
        Term::Branch(branch) => {
            branch.arms.iter().for_each(|arm| {
                codegen_block(ctx, &arm.0);
                ctx.target.write_line("if c.check_condition()? {");
                ctx.target.increase_indent();
                codegen_block(ctx, &arm.1);
                ctx.target.decrease_indent();
                ctx.target.write_line("} else {");
                ctx.target.increase_indent();
            });

            branch.arms.iter().for_each(|_| {
                ctx.target.decrease_indent();
                ctx.target.write_line("}");
            });
        }
        Term::Loop(loop_t) => codegen_loop(ctx, loop_t),
    }
}

fn codegen_block(ctx: &mut CodegenContext, block: &Block) {
    block.terms.iter().for_each(|t| codegen_term(ctx, t));
}

fn codegen_func(ctx: &mut CodegenContext, name: &str, body: &Block) {
    ctx.target.write_line(&format!(
        "fn {}(c: &mut Interpreter) -> InterpreterResult {{",
        name
    ));
    ctx.target.increase_indent();
    codegen_block(ctx, body);
    ctx.target.write_line("Ok(())");
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
}

pub fn rs_codegen_module(program: &Program, main_namespace: NamespaceId, main: &Block) {
    let mut ctx = CodegenContext {
        namespace: 0,
        program,
        target: Default::default(),
    };

    for (id, ast) in program.namespaces.iter().enumerate() {
        ctx.namespace = id;
        for (_, func) in ast.functions.iter() {
            let name = &ctx.get_scoped_name(&func.name);
            codegen_func(&mut ctx, name, &func.body);
        }
    }

    ctx.namespace = main_namespace;
    codegen_func(&mut ctx, "main_body", main);

    let definitions = {
        let definition_start = INTERPRETER
            .find("// Codegen Interpreter Start")
            .expect("Should contain definitions");

        let definition_end = INTERPRETER
            .find("// Codegen Interpreter End")
            .expect("Should contain definitions");

        DEFS.to_string().replace(
            "// Interpreter API",
            &INTERPRETER[definition_start..definition_end]
                .replace("Value<'a>", "Value")
                .replace("'a", "'static"),
        )
    };

    let intrinsics = {
        let definition_start = INTRINSICS
            .find("// Codegen Intrinsics Start")
            .expect("Should contain definitions");

        let definition_end = INTRINSICS
            .find("// Codegen Intrinsics End")
            .expect("Should contain definitions");
        &INTRINSICS[definition_start..definition_end]
    };

    println!(
        "{definitions}\n{intrinsics}{}
fn main() -> InterpreterResult {{
  let mut c = Interpreter::new();
  main_body(&mut c)?;
  c.print()?;
  Ok(())
}}",
        ctx.target.into_string()
    );
}
