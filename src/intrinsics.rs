use std::{collections::HashMap, ops::Not, sync::OnceLock};

use crate::{
    interpreter::{Interpreter, InterpreterResult},
    lang::{Arity, Type, Value},
};

type Intrinsic = fn(&mut Interpreter) -> InterpreterResult;

fn plus(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a + b)
}

fn minus(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a - b)
}

fn times(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a * b)
}

fn divide(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a / b)
}

fn modulo(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a % b)
}

fn or(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2()?;
    i.push(if a.is_truthy() { a } else { b })
}

fn and(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2()?;
    i.push(if a.is_truthy() { b } else { a })
}

fn swap(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2()?;
    i.push2(b, a)
}
fn dup(i: &mut Interpreter) -> InterpreterResult {
    let v = i.take()?;
    i.push2(v.clone(), v)
}
fn over(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2()?;
    i.push3(a.clone(), b, a)
}

fn rot(i: &mut Interpreter) -> InterpreterResult {
    let (a, b, c) = i.take3()?;
    i.push3(b, c, a)
}

fn drop(i: &mut Interpreter) -> InterpreterResult {
    let _ = i.take()?;
    Ok(())
}

fn greater(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a > b)
}

fn less(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a < b)
}

fn not(i: &mut Interpreter) -> InterpreterResult {
    let v = i.take()?;
    i.push(v.is_truthy().not())
}

fn decrement(i: &mut Interpreter) -> InterpreterResult {
    let v = i.take_number()?;
    i.push(v - 1f64)
}

fn increment(i: &mut Interpreter) -> InterpreterResult {
    let v = i.take_number()?;
    i.push(v + 1f64)
}

fn equals(i: &mut Interpreter) -> InterpreterResult {
    match i.take2()? {
        (Value::Number(a), Value::Number(b)) => i.push(a == b),
        (Value::String(a), Value::String(b)) => i.push(a == b),
        (Value::Bool(a), Value::Bool(b)) => i.push(a == b),
        _ => Err("Invalid types for ="),
    }
}

pub fn get_intrinsic(name: &str) -> Option<&'static Intrinsic> {
    get_intrinsics().get(name)
}

type Intrinsics = HashMap<String, Intrinsic>;

pub fn get_intrinsic_data() -> Vec<(&'static str, Arity, Intrinsic)> {
    vec![
        ("+", Arity::number_binary(), plus),
        ("-", Arity::number_binary(), minus),
        ("*", Arity::number_binary(), times),
        ("/", Arity::number_binary(), divide),
        ("%", Arity::number_binary(), modulo),
        ("||", Arity::generic_1(2, (0, 1).into()), or),
        ("&&", Arity::generic_1(2, (0, 1).into()), and),
        ("swap", Arity::generic_2(2, 0.into(), 1.into()), swap),
        ("dup", Arity::generic_2(1, 0.into(), 0.into()), dup),
        (
            "over",
            Arity::generic_3(2, 1.into(), 0.into(), 1.into()),
            over,
        ),
        (
            "rot",
            Arity::generic_3(3, 1.into(), 0.into(), 2.into()),
            rot,
        ),
        ("drop", Arity::in_out(1, 0), drop),
        (
            ">",
            Arity::binary(Type::Number, Type::Number, Type::Bool),
            greater,
        ),
        (
            "<",
            Arity::binary(Type::Number, Type::Number, Type::Bool),
            less,
        ),
        ("!", Arity::unary(Type::Unknown, Type::Bool), not),
        ("--", Arity::number_unary(), decrement),
        ("++", Arity::number_unary(), increment),
        (
            "==",
            Arity::binary(Type::Unknown, Type::Unknown, Type::Bool),
            equals,
        ),
    ]
}

static INTRINSICS: OnceLock<HashMap<String, Intrinsic>> = OnceLock::new();

fn new_intrinsics() -> Intrinsics {
    let mut i = HashMap::<String, Intrinsic>::new();
    for (name, _, intrinsic) in get_intrinsic_data() {
        i.insert(name.into(), intrinsic);
    }
    i
}

pub fn get_intrinsics() -> &'static HashMap<String, Intrinsic> {
    INTRINSICS.get_or_init(new_intrinsics)
}
