use std::sync::OnceLock;

use crate::{
    analyze::AnalysisError,
    convert::{f64_to_char, f64_to_usize, usize_to_f64},
    interpreter::{Interpreter, InterpreterResult},
    lang::{
        Arity, Type, Value,
        string::{CharString, StringApi as _},
    },
};

type Intrinsic = fn(&mut Interpreter) -> InterpreterResult;

// Codegen Intrinsics Start
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

fn pow_i(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2_numbers()?;
    i.push(a.powf(b))
}

fn or_i(i: &mut Interpreter) -> InterpreterResult {
    let (a, b) = i.take2()?;
    i.push(if a.is_truthy() { a } else { b })
}

fn and_i(i: &mut Interpreter) -> InterpreterResult {
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
    i.push(!v.is_truthy())
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
    let Some(end) = f64_to_usize(i.take_number()?) else {
        return Err("Invalid substring end index".into());
    };
    let Some(start) = f64_to_usize(i.take_number()?) else {
        return Err("Invalid substring start index".into());
    };
    let original = i.take_string()?;
    let start = start.min(original.len());
    let end = end.min(original.len()).max(start);
    i.push(Value::String(original.substring(start..end)))
}

fn join(i: &mut Interpreter) -> InterpreterResult {
    let (first, second) = i.take2()?;
    i.push(format!("{first}{second}"))
}

fn length(i: &mut Interpreter) -> InterpreterResult {
    let Some(len) = usize_to_f64(i.take_string()?.len()) else {
        return Err("String length is out of range".into());
    };
    i.push(len)
}

fn to_char(i: &mut Interpreter) -> InterpreterResult {
    let s = i.take_string()?;
    if s.len() != 1 {
        return Err("to_ascii only works on strings with length: 1".into());
    }
    let code = s[0] as u32;
    i.push(f64::from(code))
}

fn from_char(i: &mut Interpreter) -> InterpreterResult {
    let s = i.take_number()?;
    let Some(char) = f64_to_char(s) else {
        return Err("from_char only works with valid unicode codepoints".into());
    };
    i.push(Value::String(CharString::from(char)))
}

fn string_index(i: &mut Interpreter) -> InterpreterResult {
    let needle = i.take_string()?;
    let haystack = i.take_string()?;
    let location = match haystack.find(&needle) {
        Some(e) => match usize_to_f64(e) {
            Some(e) => e,
            None => return Err("String index cannot be converted to number".into()),
        },
        None => -1f64,
    };
    i.push(location)
}

fn equals(i: &mut Interpreter) -> InterpreterResult {
    match i.take2()? {
        #[expect(clippy::float_cmp)]
        (Value::Number(a), Value::Number(b)) => i.push(a == b),
        (Value::String(a), Value::String(b)) => i.push(a == b),
        (Value::Bool(a), Value::Bool(b)) => i.push(a == b),
        _ => Err("Mismatched types cannot be compared with ==".into()),
    }
}

fn print(i: &mut Interpreter) -> InterpreterResult {
    #![expect(clippy::print_stdout, reason = "print intrinsic")]
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
    let message = i.take_string()?;
    if i.take()?.is_truthy() {
        Ok(())
    } else {
        Err(format!("Assertion failed: {}", message).into())
    }
}
// Codegen Intrinsics End

fn eval_i(i: &mut Interpreter) -> InterpreterResult {
    if let Value::Address(namespace, name) = i.take()? {
        i.evaluate_name(namespace, &name)
    } else {
        Err("Expected function pointer on top of stack".into())
    }
}

type RawIntrinsic = (&'static str, Arity, Intrinsic);

pub struct IntrinsicData {
    pub name: &'static str,
    arity: Arity,
    pub func: Intrinsic,
}

impl From<RawIntrinsic> for IntrinsicData {
    fn from(value: RawIntrinsic) -> Self {
        IntrinsicData {
            name: value.0,
            arity: value.1,
            func: value.2,
        }
    }
}

static N: Type = Type::Number;
static S: Type = Type::String;
static B: Type = Type::Bool;
static U: Type = Type::Unknown;

fn get_intrinsic_data() -> IntrinsicsData {
    let i: [RawIntrinsic; _] = [
        ("+", Arity::number_binary(), plus),
        ("-", Arity::number_binary(), minus),
        ("*", Arity::number_binary(), times),
        ("/", Arity::number_binary(), divide),
        ("%", Arity::number_binary(), modulo),
        ("**", Arity::number_binary(), pow_i),
        ("||", Arity::generic_1(2, (0, 1)), or_i),
        ("&&", Arity::generic_1(2, (0, 1)), and_i),
        ("swap", Arity::generic_2(2, 0, 1), swap),
        ("dup", Arity::generic_2(1, 0, 0), dup),
        ("over", Arity::generic_3(2, 1, 0, 1), over),
        ("rot", Arity::generic_3(3, 1, 0, 2), rot),
        ("drop", (vec![Type::Unknown], vec![]).into(), drop),
        ("print", (vec![Type::Unknown], vec![]).into(), print),
        ("readline", Arity::push_two(S, B), readline),
        ("substring", (vec![N, N, S], vec![S]).into(), substring),
        ("to_char", Arity::unary(S, N), to_char),
        ("from_char", Arity::unary(N, S), from_char),
        ("index", Arity::binary(S, S, N), string_index),
        ("join", Arity::binary(U, U, S), join),
        ("length", Arity::unary(S, N), length),
        ("assert", Arity::noop().with_pop(S).with_pop(U), assert),
        ("eval", Arity::noop(), eval_i),
        (">", Arity::binary(N, N, B), greater),
        ("<", Arity::binary(N, N, B), less),
        ("!", Arity::unary(U, B), not),
        ("--", Arity::number_unary(), decrement),
        ("++", Arity::number_unary(), increment),
        ("==", Arity::binary(U, U, B), equals),
    ];

    i.into_iter()
        .map(|e| From::<RawIntrinsic>::from(e))
        .collect()
}

type IntrinsicsData = Vec<IntrinsicData>;

static INTRINSICS_DATA: OnceLock<IntrinsicsData> = OnceLock::new();

pub fn get_intrinsics() -> &'static IntrinsicsData {
    INTRINSICS_DATA.get_or_init(get_intrinsic_data)
}

static LOOKUP_TABLE_SIZE: usize = 256;

fn hash_name(name: &str) -> usize {
    let mut b = name.bytes();
    let f1 = b.next().unwrap_or_default();
    let f2 = b.next().unwrap_or_default();

    (f1 as usize * 300 + f2 as usize) % LOOKUP_TABLE_SIZE
}

fn create_lookup_table() -> Vec<Option<&'static IntrinsicData>> {
    let v = get_intrinsics();
    let mut res = Vec::<Option<&'static IntrinsicData>>::with_capacity(LOOKUP_TABLE_SIZE);

    while res.len() < LOOKUP_TABLE_SIZE {
        res.push(None);
    }

    for f in v {
        let index = hash_name(f.name);
        assert!(
            res[index].is_none(),
            "Hash collision for {:?} ({})",
            f.name,
            index
        );
        res[index] = Some(f);
    }

    res
}

static LOOKUP_TABLE: OnceLock<Vec<Option<&'static IntrinsicData>>> = OnceLock::new();

pub fn get_intrinsic(name: &str) -> Option<&'static IntrinsicData> {
    let table = LOOKUP_TABLE.get_or_init(create_lookup_table);
    let hash = hash_name(name);
    match table[hash] {
        Some(e) => {
            if e.name == name {
                Some(e)
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn get_intrinsic_codegen_name(name: &str) -> Option<&'static str> {
    Some(match get_intrinsic(name).map(|e| e.name)? {
        "+" => "plus",
        "index" => "string_index",
        "eval" => "eval_i",
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
        t => t,
    })
}

pub fn get_intrinsic_arity(name: &str) -> Result<Option<&'static Arity>, AnalysisError> {
    if name == "eval" {
        return Err(AnalysisError::IndefiniteSize);
    }

    Ok(get_intrinsic(name).map(|f| &f.arity))
}
