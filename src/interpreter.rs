use std::{
    borrow::Cow,
    io::{BufRead, StdinLock},
    ops::Not,
};

use crate::{
    intrinsics::{IntrinsicData, get_intrinsic},
    lang::{Block, Branch, Loop, OwnedValue, Term, Value},
    program::{NamespaceId, Program},
};

pub type InterpreterResult = Result<(), &'static str>;

pub struct Interpreter<'a> {
    pub stack: Vec<Value<'a>>,
    pub namespace_stack: Vec<NamespaceId>,
    pub program: &'a Program,
    input: Option<StdinLock<'static>>,
}

#[derive(Default)]
pub struct InterpreterSnapshot {
    pub stack: Vec<OwnedValue>,
}

impl<'a> Interpreter<'a> {
    #[allow(dead_code)]
    pub fn begin(program: &'a Program) -> Self {
        Self::from_snapshot(InterpreterSnapshot::default(), program)
    }

    pub fn from_snapshot(snapshot: InterpreterSnapshot, program: &'a Program) -> Self {
        Self {
            stack: snapshot.stack.into_iter().map(Into::into).collect(),
            namespace_stack: vec![],
            program,
            input: Some(std::io::stdin().lock()),
        }
    }

    pub fn execute(mut self, block: &'a Block) -> Result<InterpreterSnapshot, &'static str> {
        self.evaluate_block(block)?;
        Ok(InterpreterSnapshot {
            stack: self.stack.into_iter().map(Into::into).collect(),
        })
    }

    pub fn enable_stdin(&mut self) {
        if self.input.is_none() {
            self.input = Some(std::io::stdin().lock());
        }
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

    // Codegen Interpreter Start
    pub fn take(&mut self) -> Result<Value<'a>, &'static str> {
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

    pub fn take_string(&mut self) -> Result<Cow<'a, str>, &'static str> {
        match self.take()? {
            Value::String(v) => Ok(v),
            _ => Err("Expected string on top of stack"),
        }
    }

    pub fn push<T>(&mut self, v: T) -> InterpreterResult
    where
        Value<'a>: From<T>,
    {
        self.stack.push(v.into());
        Ok(())
    }

    pub fn push2<T1, T2>(&mut self, a: T1, b: T2) -> InterpreterResult
    where
        Value<'a>: From<T1>,
        Value<'a>: From<T2>,
    {
        self.stack.push(a.into());
        self.stack.push(b.into());
        Ok(())
    }

    pub fn push3<T1, T2, T3>(&mut self, a: T1, b: T2, c: T3) -> InterpreterResult
    where
        Value<'a>: From<T1>,
        Value<'a>: From<T2>,
        Value<'a>: From<T3>,
    {
        self.stack.push(a.into());
        self.stack.push(b.into());
        self.stack.push(c.into());
        Ok(())
    }

    pub fn take2(&mut self) -> Result<(Value<'a>, Value<'a>), &'static str> {
        let top = self.take()?;
        let second = self.take()?;
        Ok((second, top))
    }

    pub fn take3(&mut self) -> Result<(Value<'a>, Value<'a>, Value<'a>), &'static str> {
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

    // Codegen Interpreter End

    fn get_current_namespace(&self) -> usize {
        self.namespace_stack
            .last()
            .map(std::borrow::ToOwned::to_owned)
            .unwrap_or_default()
    }

    pub fn evaluate_namespaced_function(
        &mut self,
        namespace: NamespaceId,
        name: &str,
    ) -> InterpreterResult {
        let function = &self.program.namespaces[namespace].functions[name];
        let current_namespace = self.get_current_namespace();

        if namespace == current_namespace {
            self.evaluate_block(&function.body)?;
        } else {
            self.namespace_stack.push(namespace);
            self.evaluate_block(&function.body)?;
            self.namespace_stack.pop();
        }

        Ok(())
    }

    fn evaluate_block(&mut self, block: &'a Block) -> Result<(), &'static str> {
        for term in &block.terms {
            self.evaluate_term(term)?;
        }
        Ok(())
    }

    fn evaluate_branch(&mut self, b: &'a Branch) -> InterpreterResult {
        for arm in &b.arms {
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
        }

        let current_namespace = self.get_current_namespace();

        let Some((resolved_namespace, resolved_name)) =
            self.program.resolve_function(current_namespace, name)
        else {
            eprintln!("Failed to resolve function: [{:?}]", name);
            return Err("Unknown function name");
        };

        let function = &self.program.namespaces[resolved_namespace].functions[resolved_name];

        if resolved_namespace == current_namespace {
            self.evaluate_block(&function.body)?;
        } else {
            self.namespace_stack.push(resolved_namespace);
            self.evaluate_block(&function.body)?;
            self.namespace_stack.pop();
        }

        Ok(())
    }

    fn evaluate_loop(&mut self, l: &'a Loop) -> InterpreterResult {
        loop {
            match &l.pre_condition {
                None => {}
                Some(b) => {
                    self.evaluate_block(b)?;
                    if self.take()?.is_truthy().not() {
                        return Ok(());
                    }
                }
            }
            self.evaluate_block(&l.body)?;
            match &l.post_condition {
                None => {}
                Some(b) => {
                    self.evaluate_block(b)?;
                    if self.take()?.is_truthy().not() {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn evaluate_address(&mut self, name: &str) -> InterpreterResult {
        let current_namespace = self.get_current_namespace();
        self.push(Value::Address(current_namespace, name.into()))
    }

    fn evaluate_term(&mut self, term: &'a Term) -> InterpreterResult {
        match term {
            Term::String(l) => self.push(Value::String(l.into())),
            Term::Number(l) => self.push(Value::Number(*l)),
            Term::Bool(l) => self.push(Value::Bool(*l)),
            Term::Name(name) => self.evaluate_name(name),
            Term::Branch(b) => self.evaluate_branch(b),
            Term::Loop(l) => self.evaluate_loop(l),
            Term::Address(s) => self.evaluate_address(s),
        }
    }
}
