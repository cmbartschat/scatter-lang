use std::{
    iter::{Map, Peekable},
    vec::IntoIter,
};

use crate::{
    lang::{
        Block, Branch, Function, Import, ImportLocation, ImportNaming, Loop, Module, ParsedToken,
        Symbol, Term, Token,
    },
    tokenizer::{TokenizeError, tokenize},
};

type Tokens = Peekable<Map<IntoIter<ParsedToken>, fn(ParsedToken) -> Token>>;

pub type ParseResult<T> = Result<T, &'static str>;

enum BlockEndSymbol {
    CurlyClose,
    ParenOpen,
    ParenClose,
    SquareClose,
}

fn ignore_whitespace(tokens: &mut Tokens) {
    while tokens
        .peek()
        .is_some_and(|f| *f == Token::Symbol(Symbol::LineEnd))
    {
        tokens.next();
    }
}

fn assert_next_symbol(
    symbol: Symbol,
    message: &'static str,
    tokens: &mut Tokens,
) -> Result<(), &'static str> {
    ignore_whitespace(tokens);
    if tokens.next().is_some_and(|f| f == Token::Symbol(symbol)) {
        Ok(())
    } else {
        Err(message)
    }
}

fn maybe_consume_next_symbol(symbol: Symbol, tokens: &mut Tokens) -> bool {
    ignore_whitespace(tokens);
    if tokens.peek().is_some_and(|f| f == &Token::Symbol(symbol)) {
        tokens.next();
        true
    } else {
        false
    }
}

fn consume_block_terms(
    target: &mut Vec<Term>,
    tokens: &mut Tokens,
) -> Result<Option<BlockEndSymbol>, &'static str> {
    loop {
        match tokens.next() {
            Some(Token::String(l)) => target.push(Term::String(l)),
            Some(Token::Number(l)) => target.push(Term::Number(l)),
            Some(Token::Bool(l)) => target.push(Term::Bool(l)),
            Some(Token::Name(l)) => target.push(Term::Name(l)),
            Some(Token::Symbol(s)) => match s {
                Symbol::LineEnd => {}
                Symbol::Hash => return Err("Unexpected # in block"),
                Symbol::At => match tokens.peek() {
                    Some(Token::Name(n)) => {
                        target.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    _ => return Err("Expected name after @"),
                },
                Symbol::Colon => return Err("Unexpected : in block"),
                Symbol::CurlyClose => return Ok(Some(BlockEndSymbol::CurlyClose)),
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens)?)),
                Symbol::ParenClose => return Ok(Some(BlockEndSymbol::ParenClose)),
                Symbol::ParenOpen => return Ok(Some(BlockEndSymbol::ParenOpen)),
                Symbol::SquareClose => return Ok(Some(BlockEndSymbol::SquareClose)),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens)?)),
            },
            None => return Ok(None),
        };
    }
}

fn parse_condition(tokens: &mut Tokens) -> ParseResult<Block> {
    let mut condition = Block { terms: vec![] };

    match consume_block_terms(&mut condition.terms, tokens)? {
        Some(BlockEndSymbol::CurlyClose) => Err("Unexpected } in condition"),
        Some(BlockEndSymbol::ParenOpen) => Err("Unexpected ( in condition"),
        Some(BlockEndSymbol::SquareClose) => Err("Unexpected ] in condition"),
        Some(BlockEndSymbol::ParenClose) => Ok(condition),
        None => Err("Unexpected end of file in condition"),
    }
}

enum BranchArmStatus {
    Continue,
    Done,
}

fn parse_branch_arm(tokens: &mut Tokens) -> ParseResult<(Block, Block, BranchArmStatus)> {
    let condition = parse_condition(tokens)?;
    let mut block = Block { terms: vec![] };
    match consume_block_terms(&mut block.terms, tokens)? {
        Some(BlockEndSymbol::CurlyClose) => Ok((condition, block, BranchArmStatus::Done)),
        Some(BlockEndSymbol::ParenClose) => Err("Unexpected ) in branch arm"),
        Some(BlockEndSymbol::ParenOpen) => Ok((condition, block, BranchArmStatus::Continue)),
        Some(BlockEndSymbol::SquareClose) => Err("Unexpected ] in branch arm"),
        None => Err("Unexpected end of file in condition"),
    }
}

fn parse_branch(tokens: &mut Tokens) -> ParseResult<Branch> {
    let mut branch = Branch { arms: vec![] };

    assert_next_symbol(
        Symbol::ParenOpen,
        "First term of branch should be a condition",
        tokens,
    )?;

    loop {
        let (condition, body, status) = parse_branch_arm(tokens)?;
        branch.arms.push((condition, body));
        match status {
            BranchArmStatus::Continue => {}
            BranchArmStatus::Done => {
                return Ok(branch);
            }
        }
    }
}

fn parse_loop(tokens: &mut Tokens) -> ParseResult<Loop> {
    let mut loop_v = Loop {
        pre_condition: None,
        body: Block { terms: vec![] },
        post_condition: None,
    };

    if maybe_consume_next_symbol(Symbol::ParenOpen, tokens) {
        loop_v.pre_condition = Some(parse_condition(tokens)?);
    }

    match consume_block_terms(&mut loop_v.body.terms, tokens)? {
        Some(BlockEndSymbol::ParenClose) => Err("Unexpected ) in loop"),
        Some(BlockEndSymbol::CurlyClose) => Err("Unexpected } in loop"),
        Some(BlockEndSymbol::ParenOpen) => {
            loop_v.post_condition = Some(parse_condition(tokens)?);
            assert_next_symbol(
                Symbol::SquareClose,
                "Expected ] at the end of post condition",
                tokens,
            )?;
            Ok(loop_v)
        }
        Some(BlockEndSymbol::SquareClose) => Ok(loop_v),
        None => Err("Unexpected end of file in loop"),
    }
}

fn parse_function_body(tokens: &mut Tokens) -> ParseResult<Block> {
    let mut body = Block { terms: vec![] };

    match consume_block_terms(&mut body.terms, tokens)? {
        Some(BlockEndSymbol::ParenClose) => Err("Unexpected ) in function body"),
        Some(BlockEndSymbol::ParenOpen) => Err("Unexpected ( in function body"),
        Some(BlockEndSymbol::SquareClose) => Err("Unexpected ] in function body"),
        Some(BlockEndSymbol::CurlyClose) => Ok(body),
        None => Err("Unexpected end of file in function body"),
    }
}

fn parse_single_line(tokens: &mut Tokens) -> ParseResult<Block> {
    let mut target = vec![];

    while let Some(t) = tokens.next() {
        match t {
            Token::String(l) => target.push(Term::String(l)),
            Token::Number(l) => target.push(Term::Number(l)),
            Token::Bool(l) => target.push(Term::Bool(l)),
            Token::Name(l) => target.push(Term::Name(l)),
            Token::Symbol(s) => match s {
                Symbol::LineEnd => break,
                Symbol::CurlyOpen => target.push(Term::Branch(parse_branch(tokens)?)),
                Symbol::SquareOpen => target.push(Term::Loop(parse_loop(tokens)?)),
                _ => todo!(),
            },
        };
    }
    Ok(Block { terms: target })
}

fn parse_function(name: String, tokens: &mut Tokens) -> ParseResult<Function> {
    let is_multiline = match tokens.peek() {
        Some(Token::Symbol(Symbol::LineEnd)) | None => {
            return Ok(Function {
                name,
                body: Block { terms: vec![] },
            });
        }
        Some(Token::Symbol(Symbol::CurlyOpen)) => {
            tokens.next();
            true
        }
        _ => false,
    };

    if is_multiline {
        Ok(Function {
            name,
            body: parse_function_body(tokens)?,
        })
    } else {
        Ok(Function {
            name,
            body: parse_single_line(tokens)?,
        })
    }
}

fn parse_import(tokens: &mut Tokens) -> ParseResult<Import> {
    ignore_whitespace(tokens);

    let Some(first) = tokens.next() else {
        return Err("End of line during import");
    };

    let naming: ImportNaming = match first {
        Token::Name(n) if n == "*" => ImportNaming::Wildcard,
        Token::Name(n) => ImportNaming::Scoped(n),
        Token::Symbol(Symbol::CurlyOpen) => {
            let mut names = vec![];
            loop {
                match tokens.next() {
                    None => return Err("End of file during import"),
                    Some(Token::Symbol(Symbol::CurlyClose)) => break,
                    Some(Token::Name(n)) => names.push(n),
                    _ => todo!(),
                }
            }
            ImportNaming::Named(names)
        }
        _ => return Err("Unexpected expression in import"),
    };

    let path = match tokens.next() {
        Some(Token::String(s)) => s,
        _ => return Err("Expected path after import"),
    };

    Ok(Import {
        naming,
        location: ImportLocation::Relative(path),
    })
}

fn parse_module(tokens: &mut Tokens) -> ParseResult<Module> {
    let mut module = Module {
        imports: vec![],
        functions: vec![],
        body: Block { terms: vec![] },
    };

    while let Some(token) = tokens.next() {
        match token {
            Token::String(l) => module.body.terms.push(Term::String(l)),
            Token::Number(l) => module.body.terms.push(Term::Number(l)),
            Token::Bool(l) => module.body.terms.push(Term::Bool(l)),
            Token::Name(s) => {
                if maybe_consume_next_symbol(Symbol::Colon, tokens) {
                    module.functions.push(parse_function(s, tokens)?);
                } else {
                    module.body.terms.push(Term::Name(s));
                }
            }
            Token::Symbol(s) => match s {
                Symbol::LineEnd => {}
                Symbol::At => match tokens.peek() {
                    Some(Token::Name(n)) => {
                        module.body.terms.push(Term::Address(n.clone()));
                        tokens.next();
                    }
                    _ => return Err("Expected name after @"),
                },
                Symbol::Hash => module.imports.push(parse_import(tokens)?),
                Symbol::Colon => return Err("Unexpected : in module"),
                Symbol::ParenClose => return Err("Unexpected ) in module"),
                Symbol::ParenOpen => return Err("Unexpected ( in module"),
                Symbol::SquareClose => return Err("Unexpected ] in module"),
                Symbol::CurlyOpen => module.body.terms.push(Term::Branch(parse_branch(tokens)?)),
                Symbol::CurlyClose => return Err("Unexpected } in module"),
                Symbol::SquareOpen => module.body.terms.push(Term::Loop(parse_loop(tokens)?)),
            },
        };
    }

    Ok(module)
}

fn get_token_value(t: ParsedToken) -> Token {
    t.value
}

pub fn parse(source: &str) -> ParseResult<Module> {
    let mut tokens: Tokens = tokenize(source)
        .map_err(|f| match f {
            TokenizeError::UnboundedString(s) => {
                eprintln!("Unbounded string at {s:?}");
                "Unbounded string"
            }
        })?
        .into_iter()
        .map(get_token_value as fn(ParsedToken) -> Token)
        .peekable();
    parse_module(&mut tokens)
}
