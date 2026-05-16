use std::fmt;

use lalrpop_util::ParseError;

use crate::errors::{KError, KResult};
use crate::lexer::{Lexer, LexicalError, Tok};
use crate::surface::Cmd;

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    dead_code,
    unreachable_pub
)]
mod parser_impl {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);
}
use parser_impl::parser;

pub struct Parser {
    module: parser::ModuleParser,
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser").finish()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            module: parser::ModuleParser::new(),
        }
    }

    pub fn parse_module(&self, input: &str) -> KResult<Vec<Cmd>> {
        self.module
            .parse(Lexer::new(input))
            .map_err(|e| KError::Parse(format_err(&e)))
    }
}

pub fn parse(src: &str) -> KResult<Vec<Cmd>> {
    Parser::new().parse_module(src)
}

fn format_err(err: &ParseError<usize, Tok, LexicalError>) -> String {
    match err {
        ParseError::InvalidToken { location } => format!("invalid token at offset {location}"),
        ParseError::UnrecognizedEof { location, expected } => {
            format!("unexpected end of input at {location}, expected one of: {expected:?}")
        }
        ParseError::UnrecognizedToken {
            token: (start, tok, _),
            expected,
        } => format!("unexpected token {tok} at {start}, expected one of: {expected:?}"),
        ParseError::ExtraToken {
            token: (start, tok, _),
        } => format!("extra token {tok} at {start}"),
        ParseError::User { error } => error.to_string(),
    }
}
