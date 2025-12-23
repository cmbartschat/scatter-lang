use std::fmt::Debug;

#[derive(Copy, Clone, PartialEq)]
pub struct SourceLocation {
    pub character: usize,
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn start() -> Self {
        Self {
            character: 0,
            line: 0,
            column: 0,
        }
    }

    #[must_use]
    pub fn add(&self, c: char) -> Self {
        match c {
            '\n' => Self {
                character: self.character + 1,
                line: self.line + 1,
                column: 0,
            },
            _ => Self {
                character: self.character + 1,
                line: self.line,
                column: self.column + 1,
            },
        }
    }
}

impl Debug for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

#[derive(Copy, Clone)]
pub struct SourceRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl<T1, T2> From<(T1, T2)> for SourceRange
where
    T1: Into<SourceLocation>,
    T2: Into<SourceLocation>,
{
    fn from(value: (T1, T2)) -> Self {
        Self {
            start: value.0.into(),
            end: value.1.into(),
        }
    }
}

impl From<&SourceLocation> for SourceLocation {
    fn from(value: &SourceLocation) -> Self {
        *value
    }
}

impl Debug for SourceRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start.character == self.end.character {
            write!(f, "{:?}", self.start)
        } else {
            write!(
                f,
                "{:?}-{}.{}",
                self.start,
                self.end.line + 1,
                self.end.column + 1
            )
        }
    }
}
impl SourceRange {
    pub fn extract<'a>(&self, string: &'a str) -> Vec<(usize, &'a str)> {
        string
            .lines()
            .enumerate()
            .skip(self.start.line)
            .take(self.end.line - self.start.line + 1)
            .collect()
    }
}

impl SourceLocation {
    pub fn extract<'a>(&self, string: &'a str) -> Option<&'a str> {
        string.lines().nth(self.line)
    }
}
