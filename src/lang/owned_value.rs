use std::fmt::Debug;

use crate::lang::Value;

#[derive(Clone, PartialEq)]
pub enum OwnedValue {
    String(String),
    Number(f64),
    Bool(bool),
}

impl<'a> From<Value<'a>> for OwnedValue {
    fn from(value: Value) -> Self {
        match value {
            Value::String(v) => OwnedValue::String(v.into()),
            Value::Number(v) => OwnedValue::Number(v),
            Value::Bool(v) => OwnedValue::Bool(v),
        }
    }
}

impl<'a> From<&OwnedValue> for Value<'a> {
    fn from(value: &OwnedValue) -> Self {
        match value {
            OwnedValue::String(v) => Value::String(v.to_string().into()),
            OwnedValue::Number(v) => Value::Number(*v),
            OwnedValue::Bool(v) => Value::Bool(*v),
        }
    }
}

impl<'a> From<OwnedValue> for Value<'a> {
    fn from(v: OwnedValue) -> Value<'a> {
        match v {
            OwnedValue::String(v) => Value::String(v.into()),
            OwnedValue::Number(v) => Value::Number(v),
            OwnedValue::Bool(v) => Value::Bool(v),
        }
    }
}

impl Debug for OwnedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v: Value = self.into();
        Debug::fmt(&v, f)
    }
}

impl From<i32> for OwnedValue {
    fn from(value: i32) -> Self {
        Self::Number(value as f64)
    }
}

impl From<bool> for OwnedValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f64> for OwnedValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
