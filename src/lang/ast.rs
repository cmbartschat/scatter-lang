use crate::lang::SourceRange;

#[derive(Clone, Debug)]
pub enum Term {
    String(String),
    Number(f64),
    Bool(bool),
    Address(String),
    Name(String, SourceRange),
    Branch(Branch),
    Loop(Loop),
    Capture(String, SourceRange),
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0))
            | (Self::Address(l0), Self::Address(r0))
            | (Self::Name(l0, _), Self::Name(r0, _))
            | (Self::Capture(l0, _), Self::Capture(r0, _)) => l0 == r0,
            (Self::Branch(l0), Self::Branch(r0)) => l0 == r0,
            (Self::Loop(l0), Self::Loop(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl From<i32> for Term {
    fn from(value: i32) -> Self {
        Term::Number(value.into())
    }
}

impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Number(value)
    }
}

impl From<bool> for Term {
    fn from(value: bool) -> Self {
        Term::Bool(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    pub arms: Vec<(Block, Block)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Loop {
    pub pre_condition: Option<Block>,
    pub body: Block,
    pub post_condition: Option<Block>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Block {
    pub terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportNaming {
    Wildcard,
    Named(Vec<String>),
    Scoped(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportLocation {
    Relative(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub naming: ImportNaming,
    pub location: ImportLocation,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Module {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
    pub body: Block,
}
