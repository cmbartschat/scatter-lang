use std::fmt::{Debug, Write as _};

#[derive(Copy, Clone, PartialEq)]
pub enum Symbol {
    Colon,
    CurlyOpen,
    CurlyClose,
    ParenOpen,
    ParenClose,
    SquareOpen,
    SquareClose,
    LineEnd,
    Hash,
    At,
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
            Self::Colon => ':',
            Self::CurlyOpen => '{',
            Self::CurlyClose => '}',
            Self::ParenOpen => '(',
            Self::ParenClose => ')',
            Self::SquareOpen => '[',
            Self::SquareClose => ']',
            Self::Hash => '#',
            Self::At => '@',
            Self::LineEnd => '␤',
        })
    }
}
