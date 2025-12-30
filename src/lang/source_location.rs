use std::{fmt::Debug, iter::Peekable, str::Chars};

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

#[derive(Clone, PartialEq, Debug)]
pub struct SourcePositions {
    pub prev: Option<SourceLocation>,
    pub current: SourceLocation,
    pub next: Option<SourceLocation>,
}

pub struct SourceCrawler<'a> {
    chars: Peekable<Chars<'a>>,
    prev_location: Option<SourceLocation>,
    current_location: SourceLocation,
}

impl<'a> SourceCrawler<'a> {
    pub fn new(source: &'a str) -> Option<Self> {
        let mut iter = source.chars().peekable();
        let first_char = iter.peek().copied();
        first_char.map(|_| Self {
            chars: iter,
            prev_location: None,
            current_location: SourceLocation::start(),
        })
    }

    pub fn last_seen_location(&self) -> SourceLocation {
        self.current_location
    }
}

impl Iterator for SourceCrawler<'_> {
    type Item = (char, Option<char>, SourcePositions);

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.next().map(|char| {
            let next = self.chars.peek().copied();

            let res = (
                char,
                next,
                SourcePositions {
                    prev: self.prev_location,
                    current: self.current_location,
                    next: next.map(|_| self.current_location.add(char)),
                },
            );

            self.prev_location = Some(res.2.current);
            self.current_location = res.2.next.unwrap_or(res.2.current);

            res
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::{SourceCrawler, SourceLocation, SourcePositions};

    fn make_positions(
        prev: (usize, usize, usize),
        break_line: bool,
        break_line_after: bool,
    ) -> SourcePositions {
        let prev = SourceLocation {
            line: prev.0,
            column: prev.1,
            character: prev.2,
        };
        let current = if break_line {
            SourceLocation {
                character: prev.character + 1,
                line: prev.line + 1,
                column: 0,
            }
        } else {
            SourceLocation {
                character: prev.character + 1,
                line: prev.line,
                column: prev.column + 1,
            }
        };

        let next = if break_line_after {
            SourceLocation {
                character: current.character + 1,
                line: current.line + 1,
                column: 0,
            }
        } else {
            SourceLocation {
                character: current.character + 1,
                line: current.line,
                column: current.column + 1,
            }
        };

        SourcePositions {
            prev: Some(prev),
            current,
            next: Some(next),
        }
    }

    #[test]
    fn crawl_1() {
        let crawler = SourceCrawler::new(
            r"012
1
2",
        )
        .unwrap();

        let actual: Vec<_> = crawler.collect();

        let expected = &[
            (
                '0',
                Some('1'),
                SourcePositions {
                    prev: None,
                    current: SourceLocation {
                        line: 0,
                        column: 0,
                        character: 0,
                    },
                    next: Some(SourceLocation {
                        line: 0,
                        column: 1,
                        character: 1,
                    }),
                },
            ),
            ('1', Some('2'), make_positions((0, 0, 0), false, false)),
            ('2', Some('\n'), make_positions((0, 1, 1), false, false)),
            ('\n', Some('1'), make_positions((0, 2, 2), false, true)),
            ('1', Some('\n'), make_positions((0, 3, 3), true, false)),
            ('\n', Some('2'), make_positions((1, 0, 4), false, true)),
            (
                '2',
                None,
                SourcePositions {
                    prev: Some(SourceLocation {
                        line: 1,
                        column: 1,
                        character: 5,
                    }),
                    current: SourceLocation {
                        line: 2,
                        column: 0,
                        character: 6,
                    },
                    next: None,
                },
            ),
        ];

        for (i, actual_element) in actual.iter().enumerate() {
            assert_eq!(Some(actual_element), expected.get(i), "{i}");
        }

        assert_eq!(actual.len(), expected.len());
    }
}
