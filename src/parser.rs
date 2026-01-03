use std::{iter::Peekable, vec::IntoIter};

use crate::{
    lang::{
        Block, Branch, Function, Import, ImportLocation, ImportNaming, Loop, Module, ParsedToken,
        SourceLocation, SourceRange, Symbol, Term, Token,
    },
    parse_error::{
        EndOfFileError, ParseError, ParseSection, ReasonExpectingMore, UnexpectedContext,
        UnexpectedError, WrappedExpression, cannot_use_in, need_more, unclosed, unexpected_symbol,
        unexpected_symbol_in,
    },
    tokenizer::tokenize,
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

fn assert_next_symbol<T>(
    tokens: &mut Tokens,
    symbol: Symbol,
    err_if_unexpected: T,
    err_if_missing: EndOfFileError,
) -> ParseResult<ParsedToken>
where
    T: Fn(ParsedToken) -> UnexpectedError,
{
    ignore_whitespace(tokens);
    match tokens.next() {
        Some(t) if t.value == Token::Symbol(symbol) => Ok(t),
        Some(t) => Err(ParseError::Unexpected(err_if_unexpected(t))),
        None => Err(ParseError::EndOfFile(err_if_missing)),
    }
}

fn maybe_consume_next_symbol(symbol: Symbol, tokens: &mut Tokens) -> Option<ParsedToken> {
    ignore_whitespace(tokens);
    match tokens.peek() {
        Some(t) if t.value == Token::Symbol(symbol) => tokens.next(),
        Some(_) | None => None,
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
            Token::Name(l) => target.push(Term::Name(l, loc)),
            Token::Symbol(s) => match s {
                Symbol::LineEnd => {}
                Symbol::Hash | Symbol::Colon => {
                    return unexpected_symbol(s, loc.start);
                }
                Symbol::Tilde => parse_capture(tokens, target, &loc.start)?,
                Symbol::At => match tokens.peek() {
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => {
                        target.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    Some(ParsedToken {
                        loc: unexpected_loc,
                        ..
                    }) => {
                        return cannot_use_in(
                            UnexpectedContext::Address,
                            loc.start,
                            unexpected_loc.start,
                        );
                    }
                    None => {
                        return need_more(ReasonExpectingMore::Address, loc.start);
                    }
                },
                Symbol::CurlyClose => return Ok(Some((BlockEndSymbol::CurlyClose, loc))),
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::ParenClose => return Ok(Some((BlockEndSymbol::ParenClose, loc))),
                Symbol::ParenOpen => return Ok(Some((BlockEndSymbol::ParenOpen, loc))),
                Symbol::SquareClose => return Ok(Some((BlockEndSymbol::SquareClose, loc))),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens, &loc.start)?)),
            },
        }
    }
    Ok(None)
}

fn parse_condition(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Block> {
    let mut condition = Block { terms: vec![] };
    let section = ParseSection::Condition;

    match consume_block_terms(&mut condition.terms, tokens)? {
        Some((BlockEndSymbol::CurlyClose, loc)) => {
            unexpected_symbol_in(Symbol::CurlyClose, loc.start, section, *start)
        }
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            unexpected_symbol_in(Symbol::ParenOpen, loc.start, section, *start)
        }
        Some((BlockEndSymbol::SquareClose, loc)) => {
            unexpected_symbol_in(Symbol::SquareClose, loc.start, section, *start)
        }
        Some((BlockEndSymbol::ParenClose, _)) => Ok(condition),
        None => unclosed(*start, WrappedExpression::Condition),
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
    let section = ParseSection::Branch;
    match consume_block_terms(&mut block.terms, tokens)? {
        Some((BlockEndSymbol::CurlyClose, _)) => Ok((condition, block, BranchArmStatus::Done)),
        Some((BlockEndSymbol::ParenClose, loc)) => {
            unexpected_symbol_in(Symbol::ParenClose, loc.start, section, *start)
        }
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            Ok((condition, block, BranchArmStatus::Continue(loc.start)))
        }
        Some((BlockEndSymbol::SquareClose, loc)) => {
            unexpected_symbol_in(Symbol::SquareClose, loc.start, section, *start)
        }
        None => unclosed(*start, WrappedExpression::Branch),
    }
}

fn parse_branch(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Branch> {
    let mut branch = Branch { arms: vec![] };

    assert_next_symbol(
        tokens,
        Symbol::ParenOpen,
        |t| UnexpectedError::InContext {
            context: UnexpectedContext::FirstInBranch,
            context_start: *start,
            loc: t.loc.start,
        },
        EndOfFileError::ExpectedMoreAfter(ReasonExpectingMore::Branch, *start),
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

fn parse_capture(
    tokens: &mut Tokens,
    target: &mut Vec<Term>,
    start: &SourceLocation,
) -> ParseResult<()> {
    let mut captures = vec![];
    loop {
        match tokens.next() {
            Some(ParsedToken {
                value: Token::Name(s),
                loc,
            }) => {
                captures.push(Term::Capture(s, loc));
            }
            Some(ParsedToken {
                value: Token::Symbol(Symbol::Tilde),
                ..
            }) => break,
            Some(ParsedToken {
                value: Token::Symbol(s),
                loc,
            }) => {
                return unexpected_symbol_in(s, loc.start, ParseSection::Capture, *start);
            }
            Some(ParsedToken { value: _, loc }) => {
                return cannot_use_in(UnexpectedContext::Capture, *start, loc.start);
            }
            None => {
                return unclosed(*start, WrappedExpression::Capture);
            }
        }
    }

    target.extend(captures.into_iter().rev());
    Ok(())
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

    let section = ParseSection::Loop;
    match consume_block_terms(&mut loop_v.body.terms, tokens)? {
        Some((BlockEndSymbol::ParenClose, loc)) => {
            unexpected_symbol_in(Symbol::ParenClose, loc.start, section, *start)
        }
        Some((BlockEndSymbol::CurlyClose, loc)) => {
            unexpected_symbol_in(Symbol::CurlyClose, loc.start, section, *start)
        }
        Some((BlockEndSymbol::ParenOpen, loc)) => {
            loop_v.post_condition = Some(parse_condition(tokens, &loc.start)?);
            assert_next_symbol(
                tokens,
                Symbol::SquareClose,
                |t| UnexpectedError::InContext {
                    context: UnexpectedContext::AfterPostCondition,
                    context_start: *start,
                    loc: t.loc.start,
                },
                EndOfFileError::UnclosedExpression(WrappedExpression::Loop, *start),
            )?;
            Ok(loop_v)
        }
        Some((BlockEndSymbol::SquareClose, _)) => Ok(loop_v),
        None => unclosed(*start, WrappedExpression::Loop),
    }
}

fn parse_function_body(tokens: &mut Tokens, start: &SourceLocation) -> ParseResult<Block> {
    let mut body = Block { terms: vec![] };

    match consume_block_terms(&mut body.terms, tokens)? {
        Some((BlockEndSymbol::ParenClose, loc)) => unexpected_symbol(Symbol::ParenClose, loc.start),
        Some((BlockEndSymbol::ParenOpen, loc)) => unexpected_symbol(Symbol::ParenOpen, loc.start),
        Some((BlockEndSymbol::SquareClose, loc)) => {
            unexpected_symbol(Symbol::SquareClose, loc.start)
        }
        Some((BlockEndSymbol::CurlyClose, _)) => Ok(body),
        None => unclosed(*start, WrappedExpression::Function),
    }
}

fn parse_single_line(tokens: &mut Tokens) -> ParseResult<Block> {
    let mut target = vec![];

    while let Some(ParsedToken { value: t, loc }) = tokens.next() {
        match t {
            Token::String(l) => target.push(Term::String(l)),
            Token::Number(l) => target.push(Term::Number(l)),
            Token::Bool(l) => target.push(Term::Bool(l)),
            Token::Name(l) => target.push(Term::Name(l, loc)),
            Token::Symbol(s) => match s {
                Symbol::LineEnd => break,
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens, &loc.start)?)),
                s @ (Symbol::Colon
                | Symbol::CurlyClose
                | Symbol::ParenOpen
                | Symbol::ParenClose
                | Symbol::SquareClose
                | Symbol::Hash) => {
                    return unexpected_symbol(s, loc.start);
                }
                Symbol::Tilde => parse_capture(tokens, &mut target, &loc.start)?,
                Symbol::At => match tokens.peek() {
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => {
                        target.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    Some(ParsedToken {
                        loc: unexpected_loc,
                        ..
                    }) => {
                        return cannot_use_in(
                            UnexpectedContext::Address,
                            loc.start,
                            unexpected_loc.start,
                        );
                    }
                    None => {
                        return need_more(ReasonExpectingMore::Address, loc.start);
                    }
                },
            },
        }
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
        return need_more(ReasonExpectingMore::ImportName, *start);
    };

    let naming: ImportNaming = match first {
        Token::Name(n) if n == "*" => ImportNaming::Wildcard,
        Token::Name(n) => ImportNaming::Scoped(n),
        Token::Symbol(Symbol::CurlyOpen) => {
            let mut names = vec![];
            loop {
                match tokens.next() {
                    None => {
                        return unclosed(first_loc.start, WrappedExpression::ImportNameList);
                    }
                    Some(ParsedToken {
                        value: Token::Symbol(Symbol::CurlyClose),
                        ..
                    }) => break,
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => names.push(n),
                    Some(a) => {
                        return cannot_use_in(
                            UnexpectedContext::ImportNameList,
                            *start,
                            a.loc.start,
                        );
                    }
                }
            }
            ImportNaming::Named(names)
        }
        Token::String(_) | Token::Number(_) | Token::Bool(_) | Token::Symbol(_) => {
            return cannot_use_in(UnexpectedContext::ImportNaming, *start, first_loc.start);
        }
    };

    match tokens.next() {
        Some(ParsedToken {
            value: Token::String(s),
            ..
        }) => Ok(Import {
            naming,
            location: ImportLocation::Relative(s),
        }),
        Some(ParsedToken { loc, .. }) => {
            cannot_use_in(UnexpectedContext::ImportPath, *start, loc.start)
        }
        None => need_more(ReasonExpectingMore::ImportPath, *start),
    }
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
                    module.body.terms.push(Term::Name(s, loc));
                }
            }
            Token::Symbol(s) => match s {
                Symbol::LineEnd => {}
                Symbol::Tilde => parse_capture(tokens, &mut module.body.terms, &loc.start)?,
                Symbol::At => match tokens.peek() {
                    Some(ParsedToken {
                        value: Token::Name(n),
                        ..
                    }) => {
                        module.body.terms.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    Some(ParsedToken {
                        loc: unexpected_loc,
                        ..
                    }) => {
                        return cannot_use_in(
                            UnexpectedContext::Address,
                            loc.start,
                            unexpected_loc.start,
                        );
                    }
                    None => {
                        return need_more(ReasonExpectingMore::Address, loc.start);
                    }
                },
                Symbol::Hash => module.imports.push(parse_import(tokens, &loc.start)?),
                Symbol::Colon
                | Symbol::ParenClose
                | Symbol::ParenOpen
                | Symbol::SquareClose
                | Symbol::CurlyClose => return unexpected_symbol(s, loc.start),
                Symbol::CurlyOpen => module
                    .body
                    .terms
                    .push(Term::Branch(parse_branch(tokens, &loc.start)?)),
                Symbol::SquareOpen => module
                    .body
                    .terms
                    .push(Term::Loop(parse_loop(tokens, &loc.start)?)),
            },
        }
    }

    Ok(module)
}

pub fn parse(source: &str) -> ParseResult<Module> {
    let mut tokens = tokenize(source)
        .map_err(ParseError::Tokenization)?
        .into_iter()
        .peekable();
    parse_module(&mut tokens)
}
