use crate::lang::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Literal(Value),
    Name(String),
    Branch(Branch),
    Loop(Loop),
}

impl From<i32> for Term {
    fn from(value: i32) -> Self {
        Term::Literal(value.into())
    }
}

impl From<f64> for Term {
    fn from(value: f64) -> Self {
        Term::Literal(value.into())
    }
}

impl From<bool> for Term {
    fn from(value: bool) -> Self {
        Term::Literal(value.into())
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

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
    pub body: Block,
}
