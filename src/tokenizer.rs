use crate::lang::{Symbol, Token, Value};

fn clear_and_push_word(tokens: &mut Vec<Token>, word: &mut String) {
    match word.parse::<f64>() {
        Ok(v) => tokens.push(Token::Literal(v.into())),
        Err(_) => match word.as_str() {
            "true" => tokens.push(Token::Literal(true.into())),
            "false" => tokens.push(Token::Literal(false.into())),
            _ => {
                if !word.is_empty() {
                    tokens.push(Token::Name(word.clone()));
                }
            }
        },
    }
    word.clear();
}

fn do_symbol(tokens: &mut Vec<Token>, word: &mut String, symbol: Symbol) {
    clear_and_push_word(tokens, word);
    tokens.push(Token::Symbol(symbol));
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = vec![];

    let mut word = String::new();

    let mut is_string = false;
    let mut escape_next = false;

    for char in source.chars() {
        if is_string {
            if escape_next {
                word.push(char);
                escape_next = false;
            } else {
                match char {
                    '"' => {
                        tokens.push(Token::Literal(Value::String(word.clone())));
                        word.clear();
                        is_string = false;
                    }
                    '\\' => {
                        escape_next = true;
                    }
                    _ => {
                        word.push(char);
                    }
                }
            }
            continue;
        }
        match char {
            '"' => {
                clear_and_push_word(&mut tokens, &mut word);
                is_string = true;
            }
            ':' => do_symbol(&mut tokens, &mut word, Symbol::Colon),
            '{' => do_symbol(&mut tokens, &mut word, Symbol::CurlyOpen),
            '}' => do_symbol(&mut tokens, &mut word, Symbol::CurlyClose),
            '(' => do_symbol(&mut tokens, &mut word, Symbol::ParenOpen),
            ')' => do_symbol(&mut tokens, &mut word, Symbol::ParenClose),
            '[' => do_symbol(&mut tokens, &mut word, Symbol::SquareOpen),
            ']' => do_symbol(&mut tokens, &mut word, Symbol::SquareClose),
            ' ' | '\n' => {
                clear_and_push_word(&mut tokens, &mut word);
            }
            c => {
                word.push(c);
            }
        }
    }

    clear_and_push_word(&mut tokens, &mut word);

    tokens
}
