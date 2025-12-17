use std::{iter::Peekable, vec::IntoIter};

use crate::{
    lang::{
        Block, Branch, Function, Import, ImportLocation, ImportNaming, Loop, Module, ParsedToken,
        SourceLocation, SourceRange, Symbol, Term, Token,
    },
    tokenizer::{TokenizeError, tokenize},
};

type Tokens = Peekable<IntoIter<ParsedToken>>;

pub type ParseResult<T> = Result<T, ParseError>;

enum BlockEndSymbol {
    CurlyClose,
    ParenOpen,
    ParenClose,
    SquareClose,
}

fn ignore_whitespace(tokens: &mut Tokens) {
    while tokens
        .peek()
        .is_some_and(|f| f.value == Token::Symbol(Symbol::LineEnd))
    {
        tokens.next();
    }
}

fn loc_error<T>(message: &'static str, loc: &SourceRange) -> ParseResult<T> {
    Err(ParseError::Location(message, loc.start))
}

fn range_error_between<T>(
    message: &'static str,
    start: &SourceLocation,
    end: &SourceRange,
) -> ParseResult<T> {
    Err(ParseError::Range(message, (*start, end.end).into()))
}

fn unclosed<T>(start: SourceLocation, message: &'static str) -> ParseResult<T> {
    Err(ParseError::UnclosedExpression(message, start))
}

fn assert_next_symbol(
    tokens: &mut Tokens,
    symbol: Symbol,
    wrong_symbol_message: &'static str,
    end_of_file_message: &'static str,
) -> ParseResult<ParsedToken> {
    ignore_whitespace(tokens);
    match tokens.next() {
        Some(t) if t.value == Token::Symbol(symbol) => Ok(t),
        Some(t) => loc_error(wrong_symbol_message, &t.loc),
        None => Err(ParseError::UnexpectedEnd(end_of_file_message)),
    }
}

fn maybe_consume_next_symbol(symbol: Symbol, tokens: &mut Tokens) -> Option<ParsedToken> {
    ignore_whitespace(tokens);
    match tokens.peek() {
        Some(t) if t.value == Token::Symbol(symbol) => tokens.next(),
        Some(_) => None,
        None => None,
    }
}

fn consume_block_terms(
    target: &mut Vec<Term>,
    tokens: &mut Tokens,
) -> ParseResult<Option<(BlockEndSymbol, SourceRange)>> {
    while let Some(ParsedToken { value: token, loc }) = tokens.next() {
        match token {
            Token::String(l) => target.push(Term::String(l)),
            Token::Number(l) => target.push(Term::Number(l)),
            Token::Bool(l) => target.push(Term::Bool(l)),
            Token::Name(l) => target.push(Term::Name(l)),
            Token::Symbol(s) => match s {
                Symbol::LineEnd => {}
                Symbol::Hash => {
                    return loc_error("Unexpected # in block", &loc);
                }
                Symbol::At => match tokens.peek() {
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => {
                        target.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    Some(ParsedToken { loc, .. }) => {
                        return loc_error("Expected name after @", loc);
                    }
                    None => return Err(ParseError::UnexpectedEnd("File should not end with @")),
                },
                Symbol::Colon => {
                    return loc_error("Unexpected : in block", &loc);
                }
                Symbol::CurlyClose => return Ok(Some((BlockEndSymbol::CurlyClose, loc))),
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::ParenClose => return Ok(Some((BlockEndSymbol::ParenClose, loc))),
                Symbol::ParenOpen => return Ok(Some((BlockEndSymbol::ParenOpen, loc))),
                Symbol::SquareClose => return Ok(Some((BlockEndSymbol::SquareClose, loc))),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens, &loc.start)?)),
            },
        };
    }
    Ok(None)
}

fn parse_condition(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Block> {
    let mut condition = Block { terms: vec![] };

    match consume_block_terms(&mut condition.terms, tokens)? {
        Some((BlockEndSymbol::CurlyClose, loc)) => {
            range_error_between("Unexpected } in condition", start, &loc)
        }
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            range_error_between("Unexpected ( in condition", start, &loc)
        }
        Some((BlockEndSymbol::SquareClose, loc)) => {
            range_error_between("Unexpected ] in condition", start, &loc)
        }
        Some((BlockEndSymbol::ParenClose, _)) => Ok(condition),
        None => unclosed(*start, "Unclosed condition"),
    }
}

enum BranchArmStatus {
    Continue(SourceLocation),
    Done,
}

fn parse_branch_arm(
    tokens: &mut Tokens,
    start: &SourceLocation,
) -> ParseResult<(Block, Block, BranchArmStatus)> {
    let condition = parse_condition(tokens, start)?;
    let mut block = Block { terms: vec![] };
    match consume_block_terms(&mut block.terms, tokens)? {
        Some((BlockEndSymbol::CurlyClose, _)) => Ok((condition, block, BranchArmStatus::Done)),
        Some((BlockEndSymbol::ParenClose, loc)) => {
            range_error_between("Unexpected ) in branch arm", start, &loc)
        }
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            Ok((condition, block, BranchArmStatus::Continue(loc.start)))
        }
        Some((BlockEndSymbol::SquareClose, loc)) => {
            range_error_between("Unexpected ] in branch arm", start, &loc)
        }
        None => unclosed(*start, "Unclosed branch"),
    }
}

fn parse_branch(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Branch> {
    let mut branch = Branch { arms: vec![] };

    assert_next_symbol(
        tokens,
        Symbol::ParenOpen,
        "First term of branch should be a condition",
        "Unclosed branch",
    )?;

    let mut start = *start;

    loop {
        let (condition, body, status) = parse_branch_arm(tokens, &start)?;
        branch.arms.push((condition, body));
        match status {
            BranchArmStatus::Continue(s) => start = s,
            BranchArmStatus::Done => {
                return Ok(branch);
            }
        }
    }
}

fn parse_loop(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Loop> {
    let mut loop_v = Loop {
        pre_condition: None,
        body: Block { terms: vec![] },
        post_condition: None,
    };

    if let Some(t) = maybe_consume_next_symbol(Symbol::ParenOpen, tokens) {
        loop_v.pre_condition = Some(parse_condition(tokens, &t.loc.start)?);
    }

    match consume_block_terms(&mut loop_v.body.terms, tokens)? {
        Some((BlockEndSymbol::ParenClose, loc)) => loc_error("Unexpected ) in loop", &loc),
        Some((BlockEndSymbol::CurlyClose, loc)) => loc_error("Unexpected } in loop", &loc),
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            loop_v.post_condition = Some(parse_condition(tokens, &loc.start)?);
            assert_next_symbol(
                tokens,
                Symbol::SquareClose,
                "Expected ] at the end of post condition",
                "Unclosed loop",
            )?;
            Ok(loop_v)
        }
        Some((BlockEndSymbol::SquareClose, _)) => Ok(loop_v),
        None => unclosed(*start, "Unclosed loop"),
    }
}

fn parse_function_body(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Block> {
    let mut body = Block { terms: vec![] };

    match consume_block_terms(&mut body.terms, tokens)? {
        Some((BlockEndSymbol::ParenClose, loc)) => loc_error("Unexpected ) in function body", &loc),
        Some((BlockEndSymbol::ParenOpen, loc)) => loc_error("Unexpected ( in function body", &loc),
        Some((BlockEndSymbol::SquareClose, loc)) => {
            loc_error("Unexpected ] in function body", &loc)
        }
        Some((BlockEndSymbol::CurlyClose, _)) => Ok(body),
        None => unclosed(*start, "Unclosed function body"),
    }
}

fn parse_single_line(tokens: &mut Tokens) -> ParseResult<Block> {
    let mut target = vec![];

    while let Some(ParsedToken { value: t, loc }) = tokens.next() {
        match t {
            Token::String(l) => target.push(Term::String(l)),
            Token::Number(l) => target.push(Term::Number(l)),
            Token::Bool(l) => target.push(Term::Bool(l)),
            Token::Name(l) => target.push(Term::Name(l)),
            Token::Symbol(s) => match s {
                Symbol::LineEnd => break,
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens, &loc.start)?)),
                _ => todo!(),
            },
        };
    }
    Ok(Block { terms: target })
}

fn parse_function(name: String, tokens: &mut Tokens) -> ParseResult<Function> {
    let multiline_start = match tokens.peek() {
        Some(ParsedToken {
            value: Token::Symbol(Symbol::LineEnd),
            ..
        })
        | None => {
            return Ok(Function {
                name,
                body: Block { terms: vec![] },
            });
        }
        Some(ParsedToken {
            value: Token::Symbol(Symbol::CurlyOpen),
            loc,
        }) => {
            let start = loc.start;
            tokens.next();
            Some(start)
        }
        _ => None,
    };

    if let Some(start) = multiline_start {
        Ok(Function {
            name,
            body: parse_function_body(tokens, &start)?,
        })
    } else {
        Ok(Function {
            name,
            body: parse_single_line(tokens)?,
        })
    }
}

fn parse_import(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Import> {
    ignore_whitespace(tokens);

    let Some(ParsedToken {
        value: first,
        loc: first_loc,
    }) = tokens.next()
    else {
        return unclosed(*start, "Incomplete import statement");
    };

    let naming: ImportNaming = match first {
        Token::Name(n) if n == "*" => ImportNaming::Wildcard,
        Token::Name(n) => ImportNaming::Scoped(n),
        Token::Symbol(Symbol::CurlyOpen) => {
            let mut names = vec![];
            loop {
                match tokens.next().map(|f| f.value) {
                    None => return unclosed(first_loc.start, "Unclosed name list"),
                    Some(Token::Symbol(Symbol::CurlyClose)) => break,
                    Some(Token::Name(n)) => names.push(n),
                    _ => todo!(),
                }
            }
            ImportNaming::Named(names)
        }
        _ => return loc_error("Unexpected expression in import", &first_loc),
    };

    let path = match tokens.next() {
        Some(ParsedToken {
            value: Token::String(s),
            ..
        }) => s,
        Some(ParsedToken { loc, .. }) => return loc_error("Expected path after import", &loc),
        None => return unclosed(*start, "Unclosed import statement"),
    };

    Ok(Import {
        naming,
        location: ImportLocation::Relative(path),
    })
}

fn parse_module(tokens: &mut Tokens) -> Result<Module, ParseError> {
    let mut module = Module {
        imports: vec![],
        functions: vec![],
        body: Block { terms: vec![] },
    };

    while let Some(ParsedToken { value: token, loc }) = tokens.next() {
        match token {
            Token::String(l) => module.body.terms.push(Term::String(l)),
            Token::Number(l) => module.body.terms.push(Term::Number(l)),
            Token::Bool(l) => module.body.terms.push(Term::Bool(l)),
            Token::Name(s) => {
                if maybe_consume_next_symbol(Symbol::Colon, tokens).is_some() {
                    module.functions.push(parse_function(s, tokens)?);
                } else {
                    module.body.terms.push(Term::Name(s));
                }
            }
            Token::Symbol(s) => match s {
                Symbol::LineEnd => {}
                Symbol::At => match tokens.peek() {
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => {
                        module.body.terms.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    Some(ParsedToken { loc, .. }) => {
                        return loc_error("Expected name after @", loc);
                    }
                    None => return Err(ParseError::UnexpectedEnd("Cannot end file after @")),
                },
                Symbol::Hash => module.imports.push(parse_import(tokens, &loc.start)?),
                Symbol::Colon => return loc_error("Unexpected : in module", &loc),
                Symbol::ParenClose => return loc_error("Unexpected ) in module", &loc),
                Symbol::ParenOpen => return loc_error("Unexpected ( in module", &loc),
                Symbol::SquareClose => return loc_error("Unexpected ] in module", &loc),
                Symbol::CurlyOpen => module
                    .body
                    .terms
                    .push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::CurlyClose => return loc_error("Unexpected } in module", &loc),
                Symbol::SquareOpen => module
                    .body
                    .terms
                    .push(Term::Loop(parse_loop(tokens, &loc.start)?)),
            },
        };
    }

    Ok(module)
}

#[derive(Debug)]
pub enum ParseError {
    UnboundedString(SourceLocation),
    UnclosedExpression(&'static str, SourceLocation),
    UnexpectedEnd(&'static str),
    Location(&'static str, SourceLocation),
    Range(&'static str, SourceRange),
}

pub fn parse(source: &str) -> ParseResult<Module> {
    let mut tokens = tokenize(source)
        .map_err(|f| match f {
            TokenizeError::UnboundedString(s) => ParseError::UnboundedString(s),
        })?
        .into_iter()
        .peekable();
    parse_module(&mut tokens)
}
