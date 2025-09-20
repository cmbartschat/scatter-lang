use std::borrow::Cow;
use std::fmt::Write;

use crate::{
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

fn codegen_loop_condition(target: &mut String, block: &Option<Block>) {
    if let Some(e) = block {
        codegen_block(target, e);
        writeln!(
            target,
            "int c; checked(check_condition(&c)); if (!c) {{break;}}"
        )
        .unwrap();
    }
}

fn codegen_loop(target: &mut String, loop_t: &Loop) {
    writeln!(target, "while(1) {{").unwrap();
    codegen_loop_condition(target, &loop_t.pre_condition);
    codegen_block(target, &loop_t.body);
    codegen_loop_condition(target, &loop_t.post_condition);
    writeln!(target, "}}").unwrap();
}

fn codegen_term(target: &mut String, term: &Term) {
    match term {
        Term::Literal(value) => match value {
            Value::String(e) => writeln!(
                target,
                "checked(push_string_literal({:?}, {}));",
                e,
                e.len()
            )
            .unwrap(),
            Value::Number(e) => writeln!(target, "checked(push_number_literal({}L));", e).unwrap(),
            Value::Bool(true) => writeln!(target, "checked(push_true_literal());").unwrap(),
            Value::Bool(false) => writeln!(target, "checked(push_false_literal());").unwrap(),
        },
        Term::Name(n) => writeln!(target, "checked({}());", maybe_mangle(n)).unwrap(),
        Term::Branch(branch) => {
            branch.arms.iter().for_each(|arm| {
                codegen_block(target, &arm.0);
                writeln!(target, "int c; checked(check_condition(&c)); if (c) {{").unwrap();
                codegen_block(target, &arm.1);
                writeln!(target, "}} else {{").unwrap();
            });

            branch.arms.iter().for_each(|_| {
                target.write_char('}').unwrap();
            });
        }
        Term::Loop(loop_t) => codegen_loop(target, loop_t),
    }
}

fn codegen_block(target: &mut String, block: &Block) {
    block.terms.iter().for_each(|t| codegen_term(target, t));
}

fn codegen_func(target: &mut String, name: &str, body: &Block) {
    writeln!(target, "int {}() {{", name).unwrap();
    codegen_block(target, body);
    writeln!(target, "return OK;").unwrap();
    writeln!(target, "}}").unwrap();
}

fn forward_declare_func(target: &mut String, func: &Function) {
    writeln!(target, "int {}();", maybe_mangle(&func.name)).unwrap();
}

pub fn codegen_module(ast: &Module) {
    let mut target = String::new();

    for func in ast.functions.iter() {
        forward_declare_func(&mut target, func);
    }

    for func in ast.functions.iter() {
        codegen_func(&mut target, &maybe_mangle(&func.name), &func.body);
    }

    codegen_func(&mut target, "main_body", &ast.body);

    println!(
        "{DEFS}{target}
int main() {{
checked(main_body());
checked(print_stack());
}}"
    );
}
