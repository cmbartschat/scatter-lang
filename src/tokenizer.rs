use crate::lang::{ParsedToken, SourceLocation, Symbol, Token};

fn clear_and_push_word(
    tokens: &mut Vec<ParsedToken>,
    state: &mut NormalParseState,
    prev_location: &Option<SourceLocation>,
    next_location: &Option<SourceLocation>,
) {
    let start = state.start;
    if let Some(next) = next_location {
        state.start = *next;
    }
    if state.word.is_empty() {
        return;
    }
    let end = prev_location.expect("Must have previous location if word is not empty");
    let word = &mut state.word;
    let token = match word.parse::<f64>() {
        Ok(v) => Token::Number(v),
        Err(_) => match word.as_str() {
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Name(word.clone()),
        },
    };
    tokens.push(token.with_range((start, end)));
    word.clear();
}

fn do_symbol(
    tokens: &mut Vec<ParsedToken>,
    state: &mut NormalParseState,
    symbol: Symbol,
    prev: &Option<SourceLocation>,
    current: &SourceLocation,
    next: &Option<SourceLocation>,
) {
    clear_and_push_word(tokens, state, prev, next);
    tokens.push(Token::Symbol(symbol).at_location(*current));
}

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

fn parse_hex(c: char) -> Option<u8> {
    let v = c as u32;
    if v >= '0' as u32 && v <= '9' as u32 {
        return Some((v - '0' as u32) as u8);
    }

    if v >= 'a' as u32 && v <= 'f' as u32 {
        return Some((v - 'a' as u32) as u8 + 10);
    }

    if v >= 'A' as u32 && v <= 'F' as u32 {
        return Some((v - 'A' as u32) as u8 + 10);
    }

    None
}

impl EscapeState {
    pub fn next(self, char: char) -> Result<(Option<Self>, Option<char>), ()> {
        let res = match self {
            EscapeState::EscapeNext => match char {
                c @ '\\' | c @ '"' | c @ '\'' => c,
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                'x' => return Ok((Some(Self::Hex(None)), None)),
                _ => {
                    return Err(());
                }
            },
            Self::Hex(x) => match x {
                Some(prev_char) => {
                    let (Some(high_value), Some(low_value)) =
                        (parse_hex(prev_char), parse_hex(char))
                    else {
                        return Err(());
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

#[derive(Debug)]
struct NormalParseState {
    start: SourceLocation,
    word: String,
}

#[derive(Debug)]
enum ParseState {
    String(StringParseState),
    Normal(NormalParseState),
    LineComment,
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
}

#[derive(Debug)]
pub enum TokenizeError {
    UnboundedString(SourceLocation),
    InvalidStringEscape(SourceLocation, char),
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

    let mut saved_locations: Option<(SourceLocation, Option<SourceLocation>)> = None;
    while let Some(char) = chars.next() {
        let (prev_location, current_location, next_location) = match saved_locations {
            Some((saved_curr, Some(saved_next))) => (
                Some(saved_curr),
                saved_next,
                chars.peek().map(|_| saved_next.add(char)),
            ),
            Some((_, None)) => {
                panic!("Shouldn't happen, we should only loop if saved_next is set");
            }
            None => (
                None,
                SourceLocation::start(),
                chars.peek().map(|_| SourceLocation::start().add(char)),
            ),
        };
        saved_locations = Some((current_location, next_location));

        match state {
            ParseState::String(ref mut s) => {
                let word = &mut s.word;
                match s.escape {
                    Some(e) => match e.next(char) {
                        Ok((next_escape, c)) => {
                            s.escape = next_escape;
                            if let Some(c) = c {
                                word.push(c);
                            }
                        }
                        Err(()) => {
                            return Err(TokenizeError::InvalidStringEscape(current_location, char));
                        }
                    },
                    None => match (char, s.delimiter) {
                        ('"', StringDelimiter::Double) | ('\'', StringDelimiter::Single) => {
                            tokens.push(
                                Token::String(word.clone()).with_range((s.start, current_location)),
                            );
                            match next_location {
                                Some(next_location) => state = ParseState::normal(next_location),
                                None => return Ok(tokens),
                            }
                        }
                        ('\\', _) => {
                            s.escape = Some(EscapeState::EscapeNext);
                        }
                        _ => {
                            word.push(char);
                        }
                    },
                }
            }
            ParseState::Normal(ref mut s) => {
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
                    _ => None,
                } {
                    do_symbol(
                        &mut tokens,
                        s,
                        sym,
                        &prev_location,
                        &current_location,
                        &next_location,
                    );
                    continue;
                }
                match char {
                    '"' => {
                        clear_and_push_word(&mut tokens, s, &prev_location, &None);
                        state = ParseState::string(current_location, StringDelimiter::Double);
                    }
                    '\'' => {
                        clear_and_push_word(&mut tokens, s, &prev_location, &None);
                        state = ParseState::string(current_location, StringDelimiter::Single);
                    }
                    '/' if chars.peek() == Some(&'/') => {
                        state = ParseState::comment();
                    }
                    ' ' => {
                        clear_and_push_word(&mut tokens, s, &prev_location, &next_location);
                    }
                    '\n' => {
                        clear_and_push_word(&mut tokens, s, &prev_location, &next_location);
                        tokens.push(Token::Symbol(Symbol::LineEnd).at_location(current_location));
                    }
                    c => {
                        s.word.push(c);
                    }
                }
            }
            ParseState::LineComment => {
                if char == '\n' {
                    state = match next_location {
                        Some(next) => ParseState::normal(next),
                        None => return Ok(tokens),
                    };
                }
            }
        };
    }

    match &mut state {
        ParseState::LineComment => {}
        ParseState::String(s) => return Err(TokenizeError::UnboundedString(s.start)),
        ParseState::Normal(s) => {
            clear_and_push_word(&mut tokens, s, &saved_locations.map(|f| f.0), &None);
        }
    };

    Ok(tokens)
}
