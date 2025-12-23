use crate::{
    convert::hex_char_to_u8,
    lang::{ParsedToken, SourceLocation, Symbol, Token},
};

#[derive(Debug, PartialEq, Copy, Clone)]
enum StringDelimiter {
    Single,
    Double,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum EscapeState {
    EscapeNext,
    Hex(Option<char>),
}

enum EscapeStateError {
    InvalidCharacter,
    InvalidHex,
}

impl EscapeState {
    pub fn next(self, char: char) -> Result<(Option<Self>, Option<char>), EscapeStateError> {
        let res = match self {
            EscapeState::EscapeNext => match char {
                c @ ('\\' | '"' | '\'') => c,
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                '\n' => return Ok((None, None)),
                'x' => return Ok((Some(Self::Hex(None)), None)),
                _ => {
                    return Err(EscapeStateError::InvalidCharacter);
                }
            },
            Self::Hex(x) => match x {
                Some(prev_char) => {
                    let (Some(high_value), Some(low_value)) =
                        (hex_char_to_u8(prev_char), hex_char_to_u8(char))
                    else {
                        return Err(EscapeStateError::InvalidHex);
                    };
                    (high_value * 16 + low_value) as char
                }
                None => return Ok((Some(Self::Hex(Some(char))), None)),
            },
        };

        Ok((None, Some(res)))
    }
}

#[derive(Debug)]
struct StringParseState {
    start: SourceLocation,
    word: String,
    delimiter: StringDelimiter,
    escape: Option<EscapeState>,
}

impl StringParseState {
    pub fn next(
        mut self,
        tokens: &mut Vec<ParsedToken>,
        char: char,
        _next_char: Option<char>,
        loc: &SourcePositions,
    ) -> Result<Option<ParseState>, TokenizeError> {
        let word = &mut self.word;
        match self.escape {
            Some(e) => match e.next(char) {
                Ok((next_escape, c)) => {
                    self.escape = next_escape;
                    if let Some(c) = c {
                        word.push(c);
                    }
                }
                Err(EscapeStateError::InvalidCharacter) => {
                    return Err(TokenizeError::InvalidStringEscapeChar(loc.current));
                }
                Err(EscapeStateError::InvalidHex) => {
                    return Err(TokenizeError::InvalidStringEscapeHex(loc.current));
                }
            },
            None => match (char, self.delimiter) {
                ('"', StringDelimiter::Double) | ('\'', StringDelimiter::Single) => {
                    tokens.push(Token::String(word.clone()).with_range((self.start, loc.current)));
                    match loc.next {
                        Some(next_location) => {
                            return Ok(Some(ParseState::normal(next_location)));
                        }
                        None => return Ok(None),
                    }
                }
                ('\\', _) => {
                    self.escape = Some(EscapeState::EscapeNext);
                }
                _ => {
                    word.push(char);
                }
            },
        }
        Ok(Some(ParseState::String(self)))
    }
}

#[derive(Debug, Copy, Clone)]
enum RangeCommentParseStage {
    Start,
    Inner,
    NextSlashEnds,
}

impl RangeCommentParseStage {
    pub fn next(self, char: char) -> Option<Self> {
        Some(match self {
            Self::Start => {
                assert!(
                    char == '*',
                    "First character of range comment should always be *"
                );
                Self::Inner
            }
            Self::Inner => {
                if char == '*' {
                    Self::NextSlashEnds
                } else {
                    Self::Inner
                }
            }
            Self::NextSlashEnds => {
                if char == '*' {
                    Self::NextSlashEnds
                } else if char == '/' {
                    return None;
                } else {
                    Self::Inner
                }
            }
        })
    }
}

#[derive(Debug)]
struct RangeCommentParseState {
    start: SourceLocation,
    stage: RangeCommentParseStage,
}

impl RangeCommentParseState {
    pub fn next(mut self, char: char, loc: &SourcePositions) -> Option<ParseState> {
        let Some(next_stage) = self.stage.next(char) else {
            return loc.next.map(ParseState::normal);
        };
        self.stage = next_stage;
        Some(ParseState::RangeComment(self))
    }
}

#[derive(Debug)]
struct NormalParseState {
    start: SourceLocation,
    word: String,
}

#[derive(Clone)]
struct SourcePositions {
    prev: Option<SourceLocation>,
    current: SourceLocation,
    next: Option<SourceLocation>,
}

impl SourcePositions {
    pub fn start(first_char: char) -> Self {
        Self {
            prev: None,
            current: SourceLocation::start(),
            next: Some(SourceLocation::start().add(first_char)),
        }
    }
    pub fn next(self, c: char) -> Self {
        Self {
            prev: Some(self.current),
            current: self
                .next
                .expect("Must have next if we're getting another char"),
            next: self.next.map(|f| f.add(c)),
        }
    }
}

impl NormalParseState {
    pub fn take(&mut self) -> Option<Token> {
        if self.word.is_empty() {
            return None;
        }
        let word = &mut self.word;
        let token = match word.parse::<f64>() {
            Ok(v) => Token::Number(v),
            Err(_) => match word.as_str() {
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                _ => Token::Name(word.clone()),
            },
        };
        word.clear();
        Some(token)
    }
    pub fn finish(&mut self, tokens: &mut Vec<ParsedToken>, prev: Option<SourceLocation>) {
        let Some(token) = self.take() else {
            return;
        };
        let end = prev.expect("Must have previous location if word is not empty");
        tokens.push(token.with_range((self.start, end)));
    }

    pub fn advance(&mut self, tokens: &mut Vec<ParsedToken>, loc: &SourcePositions) {
        self.finish(tokens, loc.prev);
        if let Some(next) = loc.next {
            self.start = next;
        }
    }

    pub fn next(
        mut self,
        tokens: &mut Vec<ParsedToken>,
        char: char,
        next_char: Option<char>,
        loc: &SourcePositions,
    ) -> ParseState {
        if let Some(sym) = match char {
            ':' => Some(Symbol::Colon),
            '@' => Some(Symbol::At),
            '{' => Some(Symbol::CurlyOpen),
            '}' => Some(Symbol::CurlyClose),
            '(' => Some(Symbol::ParenOpen),
            ')' => Some(Symbol::ParenClose),
            '[' => Some(Symbol::SquareOpen),
            ']' => Some(Symbol::SquareClose),
            '#' => Some(Symbol::Hash),
            '\n' => Some(Symbol::LineEnd),
            _ => None,
        } {
            self.advance(tokens, loc);
            tokens.push(Token::Symbol(sym).at_location(loc.current));
            return ParseState::Normal(self);
        }
        match char {
            '"' => {
                self.advance(tokens, loc);
                return ParseState::string(loc.current, StringDelimiter::Double);
            }
            '\'' => {
                self.advance(tokens, loc);
                return ParseState::string(loc.current, StringDelimiter::Single);
            }
            '/' if next_char == Some('/') => {
                self.advance(tokens, loc);
                return ParseState::comment();
            }
            '/' if next_char == Some('*') => {
                self.advance(tokens, loc);
                return ParseState::range_comment(loc.current);
            }
            ' ' => {
                self.advance(tokens, loc);
            }
            c => {
                self.word.push(c);
            }
        }

        ParseState::Normal(self)
    }
}

#[derive(Debug)]
enum ParseState {
    String(StringParseState),
    Normal(NormalParseState),
    LineComment,
    RangeComment(RangeCommentParseState),
}

impl ParseState {
    pub fn normal(start: SourceLocation) -> Self {
        Self::Normal(NormalParseState {
            start,
            word: String::new(),
        })
    }

    pub fn string(start: SourceLocation, delimiter: StringDelimiter) -> Self {
        Self::String(StringParseState {
            start,
            delimiter,
            word: String::new(),
            escape: None,
        })
    }

    pub fn comment() -> Self {
        Self::LineComment
    }

    pub fn range_comment(loc: SourceLocation) -> Self {
        Self::RangeComment(RangeCommentParseState {
            start: loc,
            stage: RangeCommentParseStage::Start,
        })
    }

    pub fn finish(
        self,
        tokens: &mut Vec<ParsedToken>,
        loc: Option<SourcePositions>,
    ) -> Result<(), TokenizeError> {
        match self {
            ParseState::LineComment => Ok(()),
            ParseState::String(s) => Err(TokenizeError::UnboundedString(s.start)),
            ParseState::Normal(mut s) => {
                s.finish(tokens, loc.map(|f| f.current));
                Ok(())
            }
            ParseState::RangeComment(s) => Err(TokenizeError::UnboundedComment(s.start)),
        }
    }

    pub fn next(
        self,
        tokens: &mut Vec<ParsedToken>,
        char: char,
        next_char: Option<char>,
        loc: &SourcePositions,
    ) -> Result<Option<ParseState>, TokenizeError> {
        match self {
            ParseState::String(s) => s.next(tokens, char, next_char, loc),
            ParseState::Normal(s) => Ok(Some(s.next(tokens, char, next_char, loc))),
            ParseState::RangeComment(s) => Ok(s.next(char, loc)),
            ParseState::LineComment => match (char, &loc.next) {
                ('\n', Some(next)) => Ok(Some(ParseState::normal(next.to_owned()))),
                ('\n', None) => Ok(None),
                _ => Ok(Some(self)),
            },
        }
    }
}

#[derive(Debug)]
pub enum TokenizeError {
    UnboundedString(SourceLocation),
    InvalidStringEscapeChar(SourceLocation),
    InvalidStringEscapeHex(SourceLocation),
    UnboundedComment(SourceLocation),
}

pub fn tokenize(source: &str) -> Result<Vec<ParsedToken>, TokenizeError> {
    let mut tokens: Vec<ParsedToken> = vec![];
    if source.is_empty() {
        return Ok(tokens);
    }
    let mut chars = source.chars().peekable();
    let mut state = if source.starts_with("#!") {
        ParseState::LineComment
    } else {
        ParseState::normal(SourceLocation::start())
    };

    let mut saved_loc: Option<SourcePositions> = None;
    while let Some(char) = chars.next() {
        let loc = match saved_loc {
            Some(l) => l.next(char),
            None => SourcePositions::start(char),
        };
        match state.next(&mut tokens, char, chars.peek().copied(), &loc) {
            Ok(None) => return Ok(tokens),
            Ok(Some(s)) => state = s,
            Err(e) => return Err(e),
        }
        saved_loc = Some(loc);
    }

    state.finish(&mut tokens, saved_loc)?;

    Ok(tokens)
}
