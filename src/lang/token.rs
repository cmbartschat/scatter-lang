use std::fmt::Debug;

use crate::lang::{SourceLocation, SourceRange, Symbol};

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
            Self::String(s) => s.fmt(f),
            Self::Number(s) => s.fmt(f),
            Self::Bool(s) => s.fmt(f),
            Self::Symbol(s) => s.fmt(f),
        }
    }
}

impl Token {
    pub fn with_range<T>(self, l: T) -> ParsedToken
    where
        T: Into<SourceRange>,
    {
        ParsedToken {
            loc: l.into(),
            value: self,
        }
    }

    pub fn at_location<T>(self, l: T) -> ParsedToken
    where
        T: Into<SourceLocation>,
        T: Clone,
    {
        ParsedToken {
            loc: (l.clone(), l).into(),
            value: self,
        }
    }
}

#[derive(Debug)]
pub struct ParsedToken {
    pub value: Token,
    #[allow(unused)]
    pub loc: SourceRange,
}
