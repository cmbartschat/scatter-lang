use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

#[derive(Clone, PartialEq)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Number(f64),
    Bool(bool),
    Address(usize, String),
}

impl Value<'_> {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Number(v) => !v.is_nan() && *v != 0f64,
            Value::Bool(b) => *b,
            Value::Address(_, _) => true,
        }
    }
}

impl From<i32> for Value<'_> {
    fn from(value: i32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value<'_> {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value<'_> {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => Debug::fmt(s, f),
            Self::Number(s) => Display::fmt(s, f),
            Self::Bool(true) => f.write_str("true"),
            Self::Bool(false) => f.write_str("false"),
            Self::Address(ns, name) => write!(f, "Fn[{ns}, {name}]"),
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::String(s) = self {
            Display::fmt(s, f)
        } else {
            Debug::fmt(&self, f)
        }
    }
}
