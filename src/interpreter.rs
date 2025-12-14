use std::{
    collections::HashMap,
    io::{BufRead, StdinLock},
    ops::Not,
};

use crate::{
    intrinsics::{IntrinsicData, get_intrinsic},
    lang::{Block, Branch, Function, Loop, Module, Term, Value},
};

pub type InterpreterResult = Result<(), &'static str>;

pub type Stack = Vec<Value>;

pub struct Interpreter {
    pub stack: Stack,
    pub functions: HashMap<String, Function>,
    input: Option<StdinLock<'static>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            functions: HashMap::new(),
            input: None,
        }
    }
    pub fn enable_stdin(&mut self) {
        if self.input.is_none() {
            self.input = Some(std::io::stdin().lock());
        }
    }

    pub fn disable_stdin(&mut self) {
        self.input = None;
    }

    pub fn readline(&mut self) -> Result<Option<String>, &'static str> {
        match &mut self.input {
            Some(e) => {
                let mut line = String::new();
                let bytes_written = e.read_line(&mut line).map_err(|_| "read_line failed")?;
                if bytes_written == 0 {
                    return Ok(None);
                }
                if line.ends_with('\n') {
                    line.pop();
                }
                Ok(Some(line))
            }
            None => Err("Cannot read line while stdin is not attached"),
        }
    }

    pub fn load(&mut self, m: &Module) -> Result<(), &'static str> {
        for function in &m.functions {
            self.functions
                .insert(function.name.clone(), function.clone());
        }

        self.evaluate_block(&m.body)
    }

    pub fn take(&mut self) -> Result<Value, &'static str> {
        match self.stack.pop() {
            Some(a) => Ok(a),
            None => Err("Stack empty"),
        }
    }

    pub fn take_number(&mut self) -> Result<f64, &'static str> {
        match self.take()? {
            Value::Number(v) => Ok(v),
            _ => Err("Expected number on top of stack"),
        }
    }

    pub fn take_string(&mut self) -> Result<String, &'static str> {
        match self.take()? {
            Value::String(v) => Ok(v),
            _ => Err("Expected string on top of stack"),
        }
    }

    pub fn push<T>(&mut self, v: T) -> InterpreterResult
    where
        Value: From<T>,
    {
        self.stack.push(v.into());
        Ok(())
    }

    pub fn push2<T1, T2>(&mut self, a: T1, b: T2) -> InterpreterResult
    where
        Value: From<T1>,
        Value: From<T2>,
    {
        self.stack.push(a.into());
        self.stack.push(b.into());
        Ok(())
    }

    pub fn push3<T1, T2, T3>(&mut self, a: T1, b: T2, c: T3) -> InterpreterResult
    where
        Value: From<T1>,
        Value: From<T2>,
        Value: From<T3>,
    {
        self.stack.push(a.into());
        self.stack.push(b.into());
        self.stack.push(c.into());
        Ok(())
    }

    pub fn take2(&mut self) -> Result<(Value, Value), &'static str> {
        let top = self.take()?;
        let second = self.take()?;
        Ok((second, top))
    }

    pub fn take3(&mut self) -> Result<(Value, Value, Value), &'static str> {
        let c = self.take()?;
        let b = self.take()?;
        let a = self.take()?;
        Ok((a, b, c))
    }

    pub fn take2_numbers(&mut self) -> Result<(f64, f64), &'static str> {
        match self.take2()? {
            (Value::Number(a), Value::Number(b)) => Ok((a, b)),
            _ => Err("Expected two numbers on top of stack"),
        }
    }

    pub fn lookup_name(&self, name: &str) -> Option<Function> {
        self.functions.get(name).cloned()
    }

    fn evaluate_block(&mut self, block: &Block) -> Result<(), &'static str> {
        for term in block.terms.iter() {
            self.evaluate_term(term)?;
        }
        Ok(())
    }

    fn evaluate_branch(&mut self, b: &Branch) -> InterpreterResult {
        for arm in b.arms.iter() {
            self.evaluate_block(&arm.0)?;
            if self.take()?.is_truthy() {
                self.evaluate_block(&arm.1)?;
                return Ok(());
            }
        }
        Ok(())
    }

    fn evaluate_name(&mut self, name: &str) -> InterpreterResult {
        if let Some(IntrinsicData { func, .. }) = get_intrinsic(name) {
            return func(self);
        };
        let Some(function) = self.lookup_name(name) else {
            eprintln!("Unsupported: {}", name);
            return Err("Unsupported operation");
        };
        self.evaluate_block(&function.body)
    }

    fn evaluate_loop(&mut self, l: &Loop) -> InterpreterResult {
        loop {
            match &l.pre_condition {
                None => {}
                Some(b) => {
                    self.evaluate_block(b)?;
                    if self.take()?.is_truthy().not() {
                        return Ok(());
                    }
                }
            };
            self.evaluate_block(&l.body)?;
            match &l.post_condition {
                None => {}
                Some(b) => {
                    self.evaluate_block(b)?;
                    if self.take()?.is_truthy().not() {
                        return Ok(());
                    }
                }
            };
        }
    }

    fn evaluate_term(&mut self, term: &Term) -> InterpreterResult {
        match term {
            Term::Literal(l) => self.push(l),
            Term::Name(name) => self.evaluate_name(name),
            Term::Branch(b) => self.evaluate_branch(b),
            Term::Loop(l) => self.evaluate_loop(l),
        }
    }
}
