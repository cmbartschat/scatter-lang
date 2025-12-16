use crate::{
    lang::{Symbol, Token},
    parser::ParseResult,
};

fn clear_and_push_word(tokens: &mut Vec<Token>, word: &mut String) {
    match word.parse::<f64>() {
        Ok(v) => tokens.push(Token::Number(v)),
        Err(_) => match word.as_str() {
            "true" => tokens.push(Token::Bool(true)),
            "false" => tokens.push(Token::Bool(false)),
            _ => {
                if !word.is_empty() {
                    tokens.push(Token::Name(word.clone()));
                }
            }
        },
    }
    word.clear();
}

fn end_line(tokens: &mut Vec<Token>, word: &mut String) {
    clear_and_push_word(tokens, word);
    tokens.push(Token::Symbol(Symbol::LineEnd));
}

fn do_symbol(tokens: &mut Vec<Token>, word: &mut String, symbol: Symbol) {
    clear_and_push_word(tokens, word);
    tokens.push(Token::Symbol(symbol));
}

struct StringParseState {
    word: String,
    escape_next: bool,
}

struct NormalParseState {
    word: String,
}

enum ParseState {
    String(StringParseState),
    Normal(NormalParseState),
    LineComment,
}

impl ParseState {
    pub fn normal() -> Self {
        Self::Normal(NormalParseState {
            word: String::new(),
        })
    }

    pub fn string() -> Self {
        Self::String(StringParseState {
            word: String::new(),
            escape_next: false,
        })
    }

    pub fn comment() -> Self {
        Self::LineComment
    }
}

pub fn tokenize(source: &str) -> ParseResult<Vec<Token>> {
    let mut tokens = vec![];
    let mut state = if source.starts_with("#!") {
        ParseState::LineComment
    } else {
        ParseState::normal()
    };
    let mut chars = source.chars().peekable();

    while let Some(char) = chars.next() {
        match state {
            ParseState::String(ref mut s) => {
                let escape_next = &mut s.escape_next;
                let word = &mut s.word;
                if *escape_next {
                    word.push(char);
                    *escape_next = false;
                } else {
                    match char {
                        '"' => {
                            tokens.push(Token::String(word.clone()));
                            state = ParseState::normal();
                        }
                        '\\' => {
                            s.escape_next = true;
                        }
                        _ => {
                            word.push(char);
                        }
                    }
                }
                continue;
            }
            ParseState::Normal(ref mut s) => {
                let word = &mut s.word;
                match char {
                    '"' => {
                        clear_and_push_word(&mut tokens, word);
                        state = ParseState::string();
                    }
                    '/' if chars.peek() == Some(&'/') => {
                        state = ParseState::comment();
                    }
                    ':' => do_symbol(&mut tokens, word, Symbol::Colon),
                    '@' => do_symbol(&mut tokens, word, Symbol::At),
                    '{' => do_symbol(&mut tokens, word, Symbol::CurlyOpen),
                    '}' => do_symbol(&mut tokens, word, Symbol::CurlyClose),
                    '(' => do_symbol(&mut tokens, word, Symbol::ParenOpen),
                    ')' => do_symbol(&mut tokens, word, Symbol::ParenClose),
                    '[' => do_symbol(&mut tokens, word, Symbol::SquareOpen),
                    ']' => do_symbol(&mut tokens, word, Symbol::SquareClose),
                    '#' => do_symbol(&mut tokens, word, Symbol::Hash),
                    ' ' => {
                        clear_and_push_word(&mut tokens, word);
                    }
                    '\n' => {
                        end_line(&mut tokens, word);
                    }
                    c => {
                        word.push(c);
                    }
                }
            }
            ParseState::LineComment => {
                if char == '\n' {
                    state = ParseState::normal();
                }
            }
        };
    }

    match &mut state {
        ParseState::LineComment => {}
        ParseState::String(_) => return Err("Unbounded string"),
        ParseState::Normal(s) => {
            end_line(&mut tokens, &mut s.word);
        }
    };

    Ok(tokens)
}
