use std::{
    fmt::Write as _,
    marker::PhantomData,
    ops::{Index, Range},
};

use crate::lang::string::api::StringApi;

// Codegen Imports End

#[derive(Clone, PartialEq)]
pub struct CharString<'a> {
    p: PhantomData<&'a ()>,
    source: Vec<char>,
}

impl StringApi<'_> for CharString<'_> {
    fn len(&self) -> usize {
        self.source.len()
    }

    fn find(&self, other: &Self) -> Option<usize> {
        let max_index = self.len().checked_sub(other.len())?;
        for i in 0..=max_index {
            let end = i + other.len();
            if self.source[i..end] == other.source {
                return Some(i);
            }
        }
        None
    }

    fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    fn substring(&self, range: Range<usize>) -> Self {
        Self {
            p: PhantomData,
            source: self.source[range].to_owned(),
        }
    }
}

impl From<&str> for CharString<'_> {
    fn from(value: &str) -> Self {
        Self {
            p: PhantomData,
            source: value.chars().collect(),
        }
    }
}

impl From<String> for CharString<'_> {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<char> for CharString<'_> {
    fn from(value: char) -> Self {
        Self {
            p: PhantomData,
            source: vec![value],
        }
    }
}

impl Index<usize> for CharString<'_> {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.source[index]
    }
}

impl Index<Range<usize>> for CharString<'_> {
    type Output = [char];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.source[index]
    }
}

impl From<CharString<'_>> for String {
    fn from(value: CharString<'_>) -> Self {
        let mut res = String::with_capacity(value.len());
        value.source.into_iter().for_each(|c| res.push(c));
        res
    }
}

impl std::fmt::Debug for CharString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('\"')?;
        self.source
            .iter()
            .try_for_each(|c| std::fmt::Display::fmt(&c.escape_debug(), f))?;
        f.write_char('\"')
    }
}

impl std::fmt::Display for CharString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.iter().try_for_each(|c| f.write_char(*c))
    }
}
