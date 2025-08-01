use std::fmt::Debug;

use crate::lang::{symbol::Symbol, value::Value};

#[derive(Clone, PartialEq)]
pub enum Token {
    Literal(Value),
    Name(String),
    Symbol(Symbol),
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(s) => s.fmt(f),
            Self::Symbol(s) => s.fmt(f),
            Self::Name(s) => f.write_str(s),
        }
    }
}
