use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

use crate::lang::Term;

#[derive(Clone, PartialEq)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Bool(bool),
}

impl<'a> Value<'a> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Number(v) => !v.is_nan() && *v != 0f64,
            Value::Bool(b) => *b,
        }
    }
}

impl<'a> From<i32> for Value<'a> {
    fn from(value: i32) -> Self {
        Value::Number(value as f64)
    }
}

impl<'a> From<&Value<'a>> for Value<'a> {
    fn from(value: &Value<'a>) -> Self {
        value.to_owned()
    }
}

impl<'a> From<bool> for Value<'a> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'a> From<f64> for Value<'a> {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Value::String(value.into())
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl<'a> Debug for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Debug::fmt(s, f),
            Self::Number(s) => Display::fmt(s, f),
            Self::Bool(true) => f.write_str("true"),
            Self::Bool(false) => f.write_str("false"),
        }
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Display::fmt(s, f),
            _ => Debug::fmt(&self, f),
        }
    }
}

impl TryFrom<&Term> for Value<'static> {
    type Error = ();

    fn try_from(value: &Term) -> Result<Self, Self::Error> {
        match value {
            Term::String(l) => Ok(l.to_owned().into()),
            Term::Number(l) => Ok((*l).into()),
            Term::Bool(l) => Ok((*l).into()),
            Term::Name(_) => Err(()),
            Term::Branch(_) => Err(()),
            Term::Loop(_) => Err(()),
        }
    }
}

impl<'a> From<Value<'a>> for Term {
    fn from(val: Value<'a>) -> Self {
        match val {
            Value::String(cow) => Term::String(cow.into_owned()),
            Value::Number(l) => l.into(),
            Value::Bool(l) => l.into(),
        }
    }
}
