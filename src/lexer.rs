use std::ops::Range;

use logos::Logos;

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

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    Nat(u32),

    #[regex("[A-Za-z][A-Za-z0-9_']*", |lex| lex.slice().to_string())]
    Ident(String),
}

pub fn tokenize(src: &str) -> Result<Vec<(Tok, Range<usize>)>, String> {
    let mut out = Vec::new();
    let mut lex = Tok::lexer(src);
    while let Some(r) = lex.next() {
        match r {
            Ok(t) => out.push((t, lex.span())),
            Err(()) => return Err(format!("lex error at {:?}: {:?}", lex.span(), lex.slice())),
        }
    }
    Ok(out)
}
