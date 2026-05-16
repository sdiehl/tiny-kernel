use std::fmt;

use logos::{Lexer as LogosLexer, Logos};
use thiserror::Error;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t\n\r\f]+")]
#[logos(skip(r"--[^\n]*", allow_greedy = true))]
pub enum Tok {
    #[token("def")]
    Def,
    #[token("axiom")]
    Axiom,
    #[token("theorem")]
    Theorem,
    #[token("fun")]
    Fun,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("by")]
    By,
    #[token("Prop")]
    Prop,
    #[token("Type")]
    TypeKw,
    #[token("Sort")]
    SortKw,

    #[token("#check")]
    Check,
    #[token("#eval")]
    Eval,
    #[token("#print")]
    Print,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":=")]
    Assign,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token("=>")]
    FatArrow,
    #[token("->")]
    Arrow,
    #[token("_")]
    Hole,

    #[regex("[0-9]+", |lex| lex.slice().parse::<u32>().ok())]
    Nat(u32),

    #[regex("[A-Za-z][A-Za-z0-9_']*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Def => write!(f, "def"),
            Self::Axiom => write!(f, "axiom"),
            Self::Theorem => write!(f, "theorem"),
            Self::Fun => write!(f, "fun"),
            Self::Let => write!(f, "let"),
            Self::In => write!(f, "in"),
            Self::By => write!(f, "by"),
            Self::Prop => write!(f, "Prop"),
            Self::TypeKw => write!(f, "Type"),
            Self::SortKw => write!(f, "Sort"),
            Self::Check => write!(f, "#check"),
            Self::Eval => write!(f, "#eval"),
            Self::Print => write!(f, "#print"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::Assign => write!(f, ":="),
            Self::Colon => write!(f, ":"),
            Self::Semi => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::FatArrow => write!(f, "=>"),
            Self::Arrow => write!(f, "->"),
            Self::Hole => write!(f, "_"),
            Self::Nat(n) => write!(f, "{n}"),
            Self::Ident(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LexicalError {
    #[error("invalid token at byte offset {0}")]
    InvalidToken(usize),
    #[error("unknown tactic: {0}")]
    UnknownTactic(String),
    #[error("intro expects an identifier, got expression")]
    BadIntroArg,
}

#[derive(Debug)]
pub struct Lexer<'input> {
    inner: LogosLexer<'input, Tok>,
}

impl<'input> Lexer<'input> {
    #[must_use]
    pub fn new(input: &'input str) -> Self {
        Self {
            inner: Tok::lexer(input),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Tok, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.inner.next()?;
        let span = self.inner.span();
        Some(match tok {
            Ok(t) => Ok((span.start, t, span.end)),
            Err(()) => Err(LexicalError::InvalidToken(span.start)),
        })
    }
}
