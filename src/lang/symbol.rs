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

#[cfg(test)]
mod tests {
    use super::Symbol;

    #[test]
    fn debug() {
        assert_eq!(
            ":{}()[]#@␤",
            &format!(
                "{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
                Symbol::Colon,
                Symbol::CurlyOpen,
                Symbol::CurlyClose,
                Symbol::ParenOpen,
                Symbol::ParenClose,
                Symbol::SquareOpen,
                Symbol::SquareClose,
                Symbol::Hash,
                Symbol::At,
                Symbol::LineEnd,
            )
        );
    }
}
