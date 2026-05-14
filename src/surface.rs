use crate::term::{Level, Name};

#[derive(Debug, Clone)]
pub enum Expr {
    Var(Name),
    Sort(Level),
    App(Box<Self>, Box<Self>),
    Lam(Name, Option<Box<Self>>, Box<Self>),
    Pi(Name, Box<Self>, Box<Self>),
    Arrow(Box<Self>, Box<Self>),
    Let(Name, Option<Box<Self>>, Box<Self>, Box<Self>),
    Hole,
}

#[derive(Debug, Clone)]
pub enum Tactic {
    Intro(Name),
    Exact(Expr),
    Apply(Expr),
    Refl,
    Assumption,
    Sorry,
    Seq(Vec<Self>),
}

#[derive(Debug, Clone)]
pub enum Cmd {
    Def(Name, Option<Expr>, Expr),
    Axiom(Name, Expr),
    Theorem(Name, Expr, Tactic),
    Check(Expr),
    Eval(Expr),
    Print(Name),
}
