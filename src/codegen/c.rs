use crate::{
    codegen::{context::CodegenContext, target::CodegenTarget},
    lang::{Block, Function, Loop, Term},
    program::{NamespaceId, Program},
};

static DEFS: &str = include_str!("./c.h");

fn codegen_loop_condition(ctx: &mut CodegenContext, block: &Option<Block>) {
    if let Some(e) = block {
        codegen_block(ctx, e);
        ctx.target.write_line("{");
        ctx.target.increase_indent();
        ctx.target.write_line("int c;");
        ctx.target.write_line("checked(check_condition(&c));");
        ctx.target.write_line("if (!c) {");
        ctx.target.write_line("  break;");
        ctx.target.write_line("}");
        ctx.target.decrease_indent();
        ctx.target.write_line("}");
    }
}

fn codegen_loop(ctx: &mut CodegenContext, loop_t: &Loop) {
    ctx.target.write_line("while (1) {");
    ctx.target.increase_indent();
    codegen_loop_condition(ctx, &loop_t.pre_condition);
    codegen_block(ctx, &loop_t.body);
    codegen_loop_condition(ctx, &loop_t.post_condition);
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
}

fn codegen_term(ctx: &mut CodegenContext, term: &Term) {
    match term {
        Term::String(e) => ctx.target.write_line(&format!(
            "checked(push_string_literal({:?}, {}));",
            e,
            e.len()
        )),
        Term::Number(e) => ctx
            .target
            .write_line(&format!("checked(push_number_literal({}L));", e)),
        Term::Bool(true) => ctx.target.write_line("checked(push_true_literal());"),
        Term::Bool(false) => ctx.target.write_line("checked(push_false_literal());"),
        Term::Name(n) => ctx
            .target
            .write_line(&format!("checked({}());", ctx.resolve_name_reference(n))),
        Term::Address(n) => ctx.target.write_line(&format!(
            "checked(push_fn_address(&{}));",
            ctx.resolve_name_reference(n)
        )),
        Term::Branch(branch) => {
            branch.arms.iter().for_each(|arm| {
                codegen_block(ctx, &arm.0);
                ctx.target.write_line("int c;");
                ctx.target.write_line("checked(check_condition(&c));");
                ctx.target.write_line("if (c) {");
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
    if body.terms.is_empty() {
        ctx.target
            .write_line(&format!("status_t {}(void) {{ return OK; }}", name));
        return;
    }
    ctx.target
        .write_line(&format!("status_t {}(void) {{", name));
    ctx.target.increase_indent();
    codegen_block(ctx, body);
    ctx.target.write_line("return OK;");
    ctx.target.decrease_indent();
    ctx.target.write_line("}");
}

fn forward_declare_func(ctx: &mut CodegenContext, func: &Function) {
    ctx.target.write_line(&format!(
        "status_t {}(void);",
        ctx.get_scoped_name(&func.name)
    ));
}

pub fn c_codegen_module(program: &Program, main_namespace: NamespaceId, main: &Block) {
    let mut ctx = CodegenContext {
        namespace: 0,
        program,
        target: CodegenTarget::default(),
    };

    for (id, ast) in program.namespaces.iter().enumerate() {
        ctx.namespace = id;
        for func in ast.functions.values() {
            forward_declare_func(&mut ctx, func);
        }
    }

    for (id, ast) in program.namespaces.iter().enumerate() {
        ctx.namespace = id;
        for func in ast.functions.values() {
            let name = &ctx.get_scoped_name(&func.name);
            codegen_func(&mut ctx, name, &func.body);
        }
    }

    ctx.namespace = main_namespace;
    codegen_func(&mut ctx, "main_body", main);

    println!(
        "{DEFS}{}
int main(void) {{
  checked(main_body());
  checked(print_stack());
}}",
        ctx.target.into_string()
    );
}
