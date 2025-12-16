use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

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
