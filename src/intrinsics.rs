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

fn pow(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a.powf(b))
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

fn substring(i: &mut Interpreter) -> InterpreterResult {
    let (start, end) = i.take2_numbers()?;
    let original = i.take_string()?;
    let start = (start as usize).min(original.len()).max(0);
    let end = (end as usize).min(original.len()).max(start);
    i.push(&original[start..end])
}

fn join(i: &mut Interpreter) -> InterpreterResult {
    let (first, second) = i.take2()?;
    i.push(format!("{first}{second}").as_str())
}

fn length(i: &mut Interpreter) -> InterpreterResult {
    let s = i.take_string()?;
    i.push(s.len() as f64)
}

fn to_char(i: &mut Interpreter) -> InterpreterResult {
    let s = i.take_string()?;
    if s.len() != 1 {
        return Err("to_ascii only works on strings with length: 1");
    }
    let byte = s.bytes().next().unwrap();
    i.push(byte as f64)
}

fn from_char(i: &mut Interpreter) -> InterpreterResult {
    let s = i.take_number()?;
    if s.fract() != 0f64 {
        return Err("from_char only works with integers");
    }
    if !s.is_finite() {
        return Err("from_char only works with normal numbers");
    }
    let v = s as u32;
    if !(0..=u8::MAX as u32).contains(&v) {
        return Err("from_char only works with numbers 0-255");
    }
    let Some(char) = char::from_u32(v) else {
        return Err("Unexpected ");
    };
    i.push(format!("{char}"))
}

fn index(i: &mut Interpreter) -> InterpreterResult {
    let needle = i.take_string()?;
    let haystack = i.take_string()?;
    let location = haystack.find(&needle).map(|e| e as f64).unwrap_or(-1f64);
    i.push(location)
}

fn equals(i: &mut Interpreter) -> InterpreterResult {
    match i.take2()? {
        (Value::Number(a), Value::Number(b)) => i.push(a == b),
        (Value::String(a), Value::String(b)) => i.push(a == b),
        (Value::Bool(a), Value::Bool(b)) => i.push(a == b),
        _ => Err("Mismatched types cannot be compared with =="),
    }
}

fn print(i: &mut Interpreter) -> InterpreterResult {
    println!("{}", i.take()?);
    Ok(())
}

fn readline(i: &mut Interpreter) -> InterpreterResult {
    if let Some(val) = i.readline()? {
        i.push(val)?;
        i.push(Value::Bool(true))
    } else {
        i.push(Value::String("".into()))?;
        i.push(Value::Bool(false))
    }
}

fn assert(i: &mut Interpreter) -> InterpreterResult {
    let message = i.take()?;
    if !i.take()?.is_truthy() {
        eprintln!("Assertion failed: {}", message);
        Err("Assertion failed")
    } else {
        Ok(())
    }
}

pub fn get_intrinsic(name: &str) -> Option<&'static IntrinsicData> {
    get_intrinsics().get(name)
}

pub struct IntrinsicData {
    pub arity: Arity,
    pub func: Intrinsic,
}

impl From<(&'static str, Arity, Intrinsic)> for IntrinsicData {
    fn from(value: (&'static str, Arity, Intrinsic)) -> Self {
        IntrinsicData {
            arity: value.1,
            func: value.2,
        }
    }
}

static N: Type = Type::Number;
static S: Type = Type::String;
static B: Type = Type::Bool;
static U: Type = Type::Unknown;

pub fn get_intrinsic_data() -> Vec<(&'static str, Arity, Intrinsic)> {
    vec![
        ("+", Arity::number_binary(), plus),
        ("-", Arity::number_binary(), minus),
        ("*", Arity::number_binary(), times),
        ("/", Arity::number_binary(), divide),
        ("%", Arity::number_binary(), modulo),
        ("**", Arity::number_binary(), pow),
        ("||", Arity::generic_1(2, (0, 1)), or),
        ("&&", Arity::generic_1(2, (0, 1)), and),
        ("swap", Arity::generic_2(2, 0, 1), swap),
        ("dup", Arity::generic_2(1, 0, 0), dup),
        ("over", Arity::generic_3(2, 1, 0, 1), over),
        ("rot", Arity::generic_3(3, 1, 0, 2), rot),
        ("drop", Arity::in_out(1, 0), drop),
        ("print", Arity::in_out(1, 0), print),
        ("readline", Arity::push_two(S, B), readline),
        ("substring", Arity::binary(N, N, S).with_pop(S), substring),
        ("to_char", Arity::unary(S, N), to_char),
        ("from_char", Arity::unary(N, S), from_char),
        ("index", Arity::binary(S, S, N), index),
        ("join", Arity::binary(U, U, S), join),
        ("length", Arity::unary(S, N), length),
        ("assert", Arity::noop().with_pop(S).with_pop(U), assert),
        (">", Arity::binary(N, N, B), greater),
        ("<", Arity::binary(N, N, B), less),
        ("!", Arity::unary(U, B), not),
        ("--", Arity::number_unary(), decrement),
        ("++", Arity::number_unary(), increment),
        ("==", Arity::binary(U, U, B), equals),
    ]
}

type IntrinsicsData = HashMap<String, IntrinsicData>;
static INTRINSICS_DATA: OnceLock<IntrinsicsData> = OnceLock::new();

fn init_intrinsics_data() -> HashMap<String, IntrinsicData> {
    let mut i = HashMap::<String, IntrinsicData>::new();
    for d in get_intrinsic_data() {
        i.insert(d.0.into(), d.into());
    }
    i
}

pub fn get_intrinsics() -> &'static HashMap<String, IntrinsicData> {
    INTRINSICS_DATA.get_or_init(init_intrinsics_data)
}

pub fn get_c_name(name: &str) -> &str {
    match name {
        "+" => "plus",
        "index" => "string_index",
        "-" => "minus",
        "*" => "times",
        "/" => "divide",
        "%" => "modulo",
        "**" => "pow_i",
        "||" => "or_i",
        "&&" => "and_i",
        ">" => "greater",
        "<" => "less",
        "!" => "not",
        "--" => "decrement",
        "++" => "increment",
        "==" => "equals",
        _ => name,
    }
}
