#!/usr/bin/env -S cargo +nightly -Zscript --quiet
---cargo
cargo-features = ["profile-rustflags"]

[profile.dev]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = 0
strip = true
rustflags = ["-Aunused"]
---

use std::borrow::Cow;
use std::{
    fmt::Write as _,
    fmt::{Debug, Display},
    io::BufRead,
    marker::PhantomData,
    ops::{Index, Range},
};

type ExecutionError = &'static str;

type InterpreterResult = Result<(), ExecutionError>;

type Operation = fn(&mut Interpreter) -> InterpreterResult;

#[derive(Clone)]
enum Value {
    String(CharString<'static>),
    Number(f64),
    Bool(bool),
    Address(&'static Operation),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Debug::fmt(s, f),
            Self::Number(s) => Display::fmt(s, f),
            Self::Bool(true) => f.write_str("true"),
            Self::Bool(false) => f.write_str("false"),
            Self::Address(a) => f.write_str("Address"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Display::fmt(s, f),
            _ => Debug::fmt(&self, f),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Number(v) => !v.is_nan() && *v != 0f64,
            Value::Bool(b) => *b,
            Value::Address(_) => true,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&'static str> for Value {
    fn from(value: &'static str) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value.into())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value.into())
    }
}

impl From<&'static Operation> for Value {
    fn from(value: &'static Operation) -> Self {
        Self::Address(value)
    }
}

struct Interpreter {
    pub stack: Vec<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1000),
        }
    }

    pub fn readline(&mut self) -> Result<Option<String>, &'static str> {
        let mut line = String::new();
        let bytes_written = std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|_| "read_line failed")?;
        if bytes_written == 0 {
            return Ok(None);
        }
        if line.ends_with('\n') {
            line.pop();
        }
        Ok(Some(line))
    }

    // Interpreter API

    pub fn check_condition(&mut self) -> Result<bool, ExecutionError> {
        Ok(self.take()?.is_truthy())
    }

    pub fn print(&self) -> InterpreterResult {
        if !self.stack.is_empty() {
            println!("{:?}", self.stack);
        }
        Ok(())
    }
}

fn eval_i(i: &mut Interpreter) -> InterpreterResult {
    match i.take()? {
        Value::Address(f) => f(i),
        _ => Err("Expected function pointer on top of stack"),
    }
}
