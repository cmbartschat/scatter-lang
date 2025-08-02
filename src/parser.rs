use std::{iter::Peekable, vec::IntoIter};

use crate::{
    lang::{Block, Branch, Function, Loop, Module, Symbol, Term, Token},
    tokenizer::tokenize,
};

type Tokens = Peekable<IntoIter<Token>>;

pub type ParseResult<T> = Result<T, &'static str>;

enum BlockEndSymbol {
    CurlyClose,
    ParenOpen,
    ParenClose,
    SquareClose,
}

fn assert_next_symbol(
    symbol: Symbol,
    message: &'static str,
    tokens: &mut Tokens,
) -> Result<(), &'static str> {
    if tokens.next().is_some_and(|f| f == Token::Symbol(symbol)) {
        Ok(())
    } else {
        Err(message)
    }
}

fn maybe_consume_next_symbol(symbol: Symbol, tokens: &mut Tokens) -> bool {
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
            Some(Token::Literal(l)) => target.push(Term::Literal(l)),
            Some(Token::Name(l)) => target.push(Term::Name(l)),
            Some(Token::Symbol(s)) => match s {
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

fn parse_function(name: String, tokens: &mut Tokens) -> ParseResult<Function> {
    assert_next_symbol(
        Symbol::CurlyOpen,
        "Function block should begin with {",
        tokens,
    )?;

    Ok(Function {
        name,
        body: parse_function_body(tokens)?,
    })
}

fn parse_module(tokens: &mut Tokens) -> ParseResult<Module> {
    let mut module = Module {
        functions: vec![],
        body: Block { terms: vec![] },
    };

    while let Some(token) = tokens.next() {
        match token {
            Token::Literal(l) => module.body.terms.push(Term::Literal(l)),
            Token::Name(s) => {
                if maybe_consume_next_symbol(Symbol::Colon, tokens) {
                    module.functions.push(parse_function(s, tokens)?);
                } else {
                    module.body.terms.push(Term::Name(s));
                }
            }
            Token::Symbol(s) => match s {
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

pub fn parse(source: &str) -> ParseResult<Module> {
    let tokens = tokenize(source)?;
    parse_module(&mut tokens.into_iter().peekable())
}
