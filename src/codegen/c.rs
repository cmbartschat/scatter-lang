use std::borrow::Cow;

use crate::{
    codegen::target::CodegenTarget,
    intrinsics::{get_c_name, get_intrinsic},
    lang::{Block, Function, Loop, Module, Term, Value},
};

static DEFS: &str = include_str!("./c.h");

fn maybe_mangle<'a>(v: &'a str) -> Cow<'a, str> {
    if get_intrinsic(v).is_some() {
        Cow::Borrowed(get_c_name(v))
    } else {
        Cow::Owned(format!("user_fn_{}", v))
    }
}

fn codegen_loop_condition(target: &mut CodegenTarget, block: &Option<Block>) {
    if let Some(e) = block {
        codegen_block(target, e);
        target.write_line("int c;");
        target.write_line("checked(check_condition(&c));");
        target.write_line("if (!c) {");
        target.write_line("  break;");
        target.write_line("}");
    }
}

fn codegen_loop(target: &mut CodegenTarget, loop_t: &Loop) {
    target.write_line("while (1) {");
    target.increase_indent();
    codegen_loop_condition(target, &loop_t.pre_condition);
    codegen_block(target, &loop_t.body);
    codegen_loop_condition(target, &loop_t.post_condition);
    target.decrease_indent();
    target.write_line("}");
}

fn codegen_term(target: &mut CodegenTarget, term: &Term) {
    match term {
        Term::Literal(value) => match value {
            Value::String(e) => target.write_line(&format!(
                "checked(push_string_literal({:?}, {}));",
                e,
                e.len()
            )),
            Value::Number(e) => {
                target.write_line(&format!("checked(push_number_literal({}L));", e))
            }
            Value::Bool(true) => target.write_line("checked(push_true_literal());"),
            Value::Bool(false) => target.write_line("checked(push_false_literal());"),
        },
        Term::Name(n) => target.write_line(&format!("checked({}());", maybe_mangle(n))),
        Term::Branch(branch) => {
            branch.arms.iter().for_each(|arm| {
                codegen_block(target, &arm.0);
                target.write_line("int c;");
                target.write_line("checked(check_condition(&c));");
                target.write_line("if (c) {");
                target.increase_indent();
                codegen_block(target, &arm.1);
                target.decrease_indent();
                target.write_line("} else {");
                target.increase_indent();
            });

            branch.arms.iter().for_each(|_| {
                target.decrease_indent();
                target.write_line("}");
            });
        }
        Term::Loop(loop_t) => codegen_loop(target, loop_t),
    }
}

fn codegen_block(target: &mut CodegenTarget, block: &Block) {
    block.terms.iter().for_each(|t| codegen_term(target, t));
}

fn codegen_func(target: &mut CodegenTarget, name: &str, body: &Block) {
    target.write_line(&format!("int {}() {{", name));
    target.increase_indent();
    codegen_block(target, body);
    target.write_line("return OK;");
    target.decrease_indent();
    target.write_line("}");
}

fn forward_declare_func(target: &mut CodegenTarget, func: &Function) {
    target.write_line(&format!("int {}();", maybe_mangle(&func.name)));
}

pub fn c_codegen_module(ast: &Module) {
    let mut target = CodegenTarget::default();

    for func in ast.functions.iter() {
        forward_declare_func(&mut target, func);
    }

    for func in ast.functions.iter() {
        codegen_func(&mut target, &maybe_mangle(&func.name), &func.body);
    }

    codegen_func(&mut target, "main_body", &ast.body);

    println!(
        "{DEFS}{}
int main() {{
  checked(main_body());
  checked(print_stack());
}}",
        target.into_string()
    );
}
