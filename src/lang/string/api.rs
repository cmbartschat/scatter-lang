use std::{
    fmt::{Debug, Display},
    ops::{Index, Range},
};

// Codegen Imports End

pub trait StringApi<'a>
where
    Self: Debug,
    Self: Display,
    Self: From<&'a str>,
    Self: From<char>,
    Self: From<String>,
    Self: Index<usize>,
    Self: Into<String>,
{
    fn len(&self) -> usize;

    fn find(&self, other: &Self) -> Option<usize>;

    fn is_empty(&self) -> bool;

    fn substring(&self, range: Range<usize>) -> Self;
}
