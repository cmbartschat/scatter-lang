use std::borrow::Cow;

use crate::{
    lang::{SourceLocation, SourceRange, Symbol},
    tokenizer::{EscapeSequenceError, TokenizeError},
};

#[derive(Debug, PartialEq)]
pub enum WrappedExpression {
    Condition,
    Branch,
    Function,
    Loop,
    ImportNameList,
}

#[derive(Debug, PartialEq)]
pub enum ParseSection {
    Condition,
    Branch,
    Loop,
}

#[derive(Debug, PartialEq)]
pub enum ReasonExpectingMore {
    Address,
    Branch,
    ImportName,
    ImportPath,
}

#[derive(Debug, PartialEq)]
pub enum EndOfFileError {
    UnclosedExpression(WrappedExpression, SourceLocation),
    ExpectedMoreAfter(ReasonExpectingMore, SourceLocation),
}

#[derive(Debug, PartialEq)]
pub enum UnexpectedContext {
    FirstInBranch,
    Address,
    AfterPostCondition,
    ImportNameList,
    ImportNaming,
    ImportPath,
}

#[derive(Debug, PartialEq)]
pub enum UnexpectedError {
    InContext {
        context: UnexpectedContext,
        context_start: SourceLocation,
        loc: SourceLocation,
    },
    SymbolInSection {
        section: ParseSection,
        section_start: SourceLocation,
        symbol: Symbol,
        symbol_location: SourceLocation,
    },
    GeneralSymbol(Symbol, SourceLocation),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Tokenization(TokenizeError),
    EndOfFile(EndOfFileError),
    Unexpected(UnexpectedError),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn unexpected_symbol<T>(symbol: Symbol, loc: SourceLocation) -> ParseResult<T> {
    Err(ParseError::Unexpected(UnexpectedError::GeneralSymbol(
        symbol, loc,
    )))
}

pub fn unexpected_symbol_in<T>(
    symbol: Symbol,
    loc: SourceLocation,
    section: ParseSection,
    start: SourceLocation,
) -> ParseResult<T> {
    Err(ParseError::Unexpected(UnexpectedError::SymbolInSection {
        section,
        symbol_location: loc,
        section_start: start,
        symbol,
    }))
}

pub fn unclosed<T>(start: SourceLocation, section: WrappedExpression) -> ParseResult<T> {
    Err(ParseError::EndOfFile(EndOfFileError::UnclosedExpression(
        section, start,
    )))
}

pub fn need_more<T>(reason: ReasonExpectingMore, start: SourceLocation) -> ParseResult<T> {
    Err(ParseError::EndOfFile(EndOfFileError::ExpectedMoreAfter(
        reason, start,
    )))
}

pub fn cannot_use_in<T>(
    context: UnexpectedContext,
    context_start: SourceLocation,
    loc: SourceLocation,
) -> ParseResult<T> {
    Err(ParseError::Unexpected(UnexpectedError::InContext {
        context,
        context_start,
        loc,
    }))
}

type ErrorDetails = (Cow<'static, str>, SourceRange, Option<Cow<'static, str>>);

struct Details {
    inner: ErrorDetails,
}

trait IntoRange {
    fn into_range(self) -> SourceRange;
}

impl IntoRange for SourceLocation {
    fn into_range(self) -> SourceRange {
        (self, self).into_range()
    }
}

impl IntoRange for SourceRange {
    fn into_range(self) -> SourceRange {
        self
    }
}
impl IntoRange for (SourceLocation, SourceLocation) {
    fn into_range(self) -> SourceRange {
        SourceRange {
            start: self.0,
            end: self.1,
        }
    }
}

impl Details {
    pub fn message<T, E>(m: T, l: E) -> ErrorDetails
    where
        T: Into<Cow<'static, str>>,
        E: IntoRange,
    {
        Self {
            inner: (m.into(), l.into_range(), None),
        }
        .done()
    }

    pub fn full<M, L, I>(m: M, l: L, i: I) -> ErrorDetails
    where
        M: Into<Cow<'static, str>>,
        L: IntoRange,
        I: Into<Cow<'static, str>>,
    {
        (m.into(), l.into_range(), Some(i.into()))
    }

    pub fn done(self) -> ErrorDetails {
        self.inner
    }
}

impl TokenizeError {
    pub fn is_early_eof(&self) -> bool {
        match self {
            TokenizeError::UnboundedString(_source_location) => true,
            TokenizeError::InvalidEscape(..) => false,
            TokenizeError::UnboundedComment(_source_location) => false,
        }
    }

    pub fn into_details(self) -> ErrorDetails {
        match self {
            Self::UnboundedString(loc) => Details::full(
                "Unclosed string literal",
                loc,
                "String literals can span multiple lines, so an earlier string may be unclosed",
            ),
            Self::InvalidEscape(EscapeSequenceError::InvalidCharacter, loc) => Details::full(
                "Invalid string escape character",
                loc,
                r"Supported escapes are: '\n', '\r', '\t', '\0', '\xff', '\u{1f4a9}', and '\u0915'",
            ),
            Self::InvalidEscape(EscapeSequenceError::InvalidHex, loc) => Details::full(
                "Invalid hex escape pattern",
                loc,
                "Correct hex escapes look like: '\\xff'",
            ),
            Self::InvalidEscape(EscapeSequenceError::InvalidUnicode, loc) => Details::full(
                "Unexpected character in unicode escape pattern",
                loc,
                r"Unicode escape sequences contain a sequence of hex digits, such as '\u{1f4a9}' and '\u0915'",
            ),
            Self::InvalidEscape(EscapeSequenceError::EmptyUnicode, loc) => {
                Details::message("Expected at least one digit in unicode escape pattern", loc)
            }
            Self::InvalidEscape(EscapeSequenceError::TooManyUnicodeDigits, loc) => Details::full(
                "Exceeded maximum unicode sequence length",
                loc,
                "Unicode codepoints do not exceed U+10FFFF which is only 6 digits",
            ),
            Self::InvalidEscape(EscapeSequenceError::OutOfUnicodeRange, loc) => Details::full(
                "Unicode escape pattern is outside the allowable range",
                loc,
                "Unicode codepoints range from U+0000 to U+10FFFF, with some invalid ranges in between",
            ),
            Self::UnboundedComment(loc) => Details::full(
                "End of input reached before ending comment",
                loc,
                "Multiline comments are closed with: */",
            ),
        }
    }
}

impl EndOfFileError {
    pub fn into_details(self) -> ErrorDetails {
        match self {
            Self::UnclosedExpression(expression, loc) => {
                let (section, info) = match expression {
                    WrappedExpression::Condition => ("condition", "Conditions are closed using: )"),
                    WrappedExpression::Branch => {
                        ("branch", "Branch statements are closed using: }")
                    }
                    WrappedExpression::Function => {
                        ("function", "Function bodies are closed using: }")
                    }
                    WrappedExpression::Loop => ("loop", "Loops are closed using ]"),
                    WrappedExpression::ImportNameList => (
                        "import name list",
                        "Import name lists are closed using }, like: # {f1 f2} \"./file.sl\"",
                    ),
                };
                Details::full(
                    format!("End of file reached before close of {section}"),
                    loc,
                    info,
                )
            }
            Self::ExpectedMoreAfter(reason, loc) => match reason {
                ReasonExpectingMore::Address => Details::full(
                    "Incomplete function pointer expression",
                    loc,
                    "A function pointer (@) must be followed directly by a function name",
                ),
                ReasonExpectingMore::Branch => Details::full(
                    "Incomplete branch expression",
                    loc,
                    "End a branch expression with: }",
                ),
                ReasonExpectingMore::ImportName => Details::full(
                    "Incomplete import statement",
                    loc,
                    "An import (#) must specify a name, wildcard, or set of functions to import",
                ),
                ReasonExpectingMore::ImportPath => Details::full(
                    "Incomplete import statement",
                    loc,
                    "An import (#) must specify a relative path: # * \"./file.sl\"",
                ),
            },
        }
    }
}

impl UnexpectedError {
    pub fn into_details(self) -> ErrorDetails {
        match self {
            Self::InContext {
                context,
                loc,
                context_start,
            } => match context {
                UnexpectedContext::FirstInBranch => Details::full(
                    "Branch must start with a condition",
                    (context_start, loc),
                    "Create a condition inside this branch statement with: {(condition) ... }",
                ),
                UnexpectedContext::Address => Details::full(
                    "Invalid function pointer",
                    context_start,
                    "A function pointer (@) must be followed directly by a function name",
                ),
                UnexpectedContext::AfterPostCondition => Details::full(
                    "Unexpected expression after loop's post condition",
                    (context_start, loc),
                    "If a loop contains a post condition, it must be the last statement before the closing ]",
                ),
                UnexpectedContext::ImportNameList => Details::full(
                    "Unexpected expression in import name list",
                    (context_start, loc),
                    "Name lists should include only names separated by spaces, such as: # {name list} \"./file1.sl\"",
                ),
                UnexpectedContext::ImportNaming => Details::full(
                    "Unexpected expression in import",
                    (context_start, loc),
                    "The first expression after # must follow the format: *, name, or {name list}",
                ),
                UnexpectedContext::ImportPath => Details::full(
                    "Invalid import path",
                    (context_start, loc),
                    "The second expression after # must be a relative path like: # file1 \"./file1.sl\"",
                ),
            },
            Self::SymbolInSection {
                section,
                section_start,
                symbol,
                symbol_location,
            } => {
                let section_name = match section {
                    ParseSection::Condition => "condition",
                    ParseSection::Branch => "branch",
                    ParseSection::Loop => "loop",
                };

                let message = format!("Unexpected {:?} in {section_name}", symbol);
                Details::message(message, (section_start, symbol_location))
            }
            Self::GeneralSymbol(symbol, loc) => {
                let message = format!("Unexpected symbol: {symbol:?}");
                let info = match symbol {
                    Symbol::Colon => ": is only used at the top level to label functions",
                    Symbol::CurlyClose => {
                        "The } symbol is only used to close branch or function blocks"
                    }
                    Symbol::ParenClose => "The ) symbol is only used to close condition blocks",
                    Symbol::SquareClose => "The ] symbol is only used to close loops",
                    Symbol::Hash => "The # symbol can only be used at the top level for imports",
                    Symbol::CurlyOpen
                    | Symbol::ParenOpen
                    | Symbol::SquareOpen
                    | Symbol::LineEnd
                    | Symbol::At => return Details::message(message, loc),
                };
                Details::full(message, loc, info)
            }
        }
    }
}

impl ParseError {
    pub fn is_early_eof(&self) -> bool {
        match self {
            Self::EndOfFile(_) => true,
            Self::Tokenization(e) => e.is_early_eof(),
            Self::Unexpected(_) => false,
        }
    }

    pub fn into_details(self) -> ErrorDetails {
        match self {
            Self::Tokenization(e) => e.into_details(),
            Self::EndOfFile(e) => e.into_details(),
            Self::Unexpected(e) => e.into_details(),
        }
    }
}
