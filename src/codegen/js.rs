use crate::{
    codegen::{
        context::{CodegenContext, CodegenResult, CodegenResultG},
        target::CodegenTarget,
    },
    lang::{Block, Loop, Term},
    program::{NamespaceId, Program},
};

static DEFS: &str = include_str!("./js.js");

fn codegen_loop_condition(ctx: &mut CodegenContext, block: Option<&Block>) -> CodegenResult {
    if let Some(e) = block {
        codegen_block(ctx, e)?;
        ctx.target.write_line("if (!checkCondition()) {");
        ctx.target.write_line("  break");
        ctx.target.write_line("}");
    }
    Ok(())
}

fn codegen_loop(ctx: &mut CodegenContext, loop_t: &Loop) -> CodegenResult {
    ctx.target.write_line("while (1) {");
    ctx.target.increase_indent();
    codegen_loop_condition(ctx, loop_t.pre_condition.as_ref())?;
    codegen_block(ctx, &loop_t.body)?;
    codegen_loop_condition(ctx, loop_t.post_condition.as_ref())?;
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
    Ok(())
}

fn codegen_term(ctx: &mut CodegenContext, term: &Term) -> CodegenResult {
    match term {
        Term::String(e) => ctx.target.write_line(&format!("push({:?})", e)),
        Term::Number(e) => ctx.target.write_line(&format!("push({})", e)),
        Term::Bool(true) => ctx.target.write_line("push(true)"),
        Term::Bool(false) => ctx.target.write_line("push(false)"),
        Term::Address(name) => ctx
            .target
            .write_line(&format!("push({})", ctx.resolve_name(name)?)),
        Term::Name(n, _) => ctx
            .target
            .write_line(&format!("{}()", ctx.resolve_name(n)?)),
        Term::Branch(branch) => {
            branch.arms.iter().try_for_each(|arm| -> CodegenResult {
                codegen_block(ctx, &arm.0)?;
                ctx.target.write_line("if (checkCondition()) {");
                ctx.target.increase_indent();
                codegen_block(ctx, &arm.1)?;
                ctx.target.decrease_indent();
                ctx.target.write_line("} else {");
                ctx.target.increase_indent();
                Ok(())
            })?;

            branch.arms.iter().for_each(|_| {
                ctx.target.decrease_indent();
                ctx.target.write_line("}");
            });
        }
        Term::Loop(loop_t) => codegen_loop(ctx, loop_t)?,
    }
    Ok(())
}

fn codegen_block(ctx: &mut CodegenContext, block: &Block) -> CodegenResult {
    block.terms.iter().try_for_each(|t| codegen_term(ctx, t))
}

fn codegen_func(ctx: &mut CodegenContext, name: &str, body: &Block) -> CodegenResult {
    ctx.target.write_line(&format!("function {}() {{", name));
    ctx.target.increase_indent();
    codegen_block(ctx, body)?;
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
    Ok(())
}

pub fn js_codegen_module(
    program: &Program,
    main_namespace: NamespaceId,
    main: &Block,
) -> CodegenResultG<String> {
    let mut ctx = CodegenContext {
        namespace: 0,
        program,
        target: CodegenTarget::default(),
    };

    ctx.target.write_line(DEFS);

    for (id, ast) in program.namespaces.iter().enumerate() {
        ctx.namespace = id;
        for func in ast.functions.values() {
            let name = &ctx.get_scoped_name(&func.name);
            codegen_func(&mut ctx, name, &func.body)?;
        }
    }

    ctx.namespace = main_namespace;
    codegen_func(&mut ctx, "main_body", main)?;

    ctx.target.write_line(
        "try {
  main_body()
  printStack()
} catch (err) {
   console.error(err)
}",
    );

    Ok(ctx.target.into_string())
}
