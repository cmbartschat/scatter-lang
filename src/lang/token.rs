use std::fmt::Debug;

use crate::lang::symbol::Symbol;

#[derive(Clone, PartialEq)]
pub enum Token {
    String(String),
    Number(f64),
    Bool(bool),
    Name(String),
    Symbol(Symbol),
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(s) => f.write_str(s),
            v => v.fmt(f),
        }
    }
}
