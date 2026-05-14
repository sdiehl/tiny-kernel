use crate::errors::{KError, KResult};
use crate::lexer::{tokenize, Tok};
use crate::surface::{Cmd, Expr, Tactic};
use crate::term::Level;

#[derive(Debug)]
pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    pub fn new(src: &str) -> KResult<Self> {
        let toks = tokenize(src).map_err(KError::Parse)?;
        Ok(Self {
            toks: toks.into_iter().map(|(t, _)| t).collect(),
            pos: 0,
        })
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.peek() == Some(t) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &Tok) -> KResult<()> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(KError::Parse(format!(
                "expected {t:?}, got {:?}",
                self.peek()
            )))
        }
    }

    fn ident(&mut self) -> KResult<String> {
        match self.bump() {
            Some(Tok::Ident(n)) => Ok(n),
            t => Err(KError::Parse(format!("expected identifier, got {t:?}"))),
        }
    }

    pub fn cmds(&mut self) -> KResult<Vec<Cmd>> {
        let mut out = Vec::new();
        while self.peek().is_some() {
            out.push(self.cmd()?);
        }
        Ok(out)
    }

    pub fn cmd(&mut self) -> KResult<Cmd> {
        match self.bump() {
            Some(Tok::Def) => {
                let n = self.ident()?;
                let ty = if self.eat(&Tok::Colon) {
                    Some(self.expr()?)
                } else {
                    None
                };
                self.expect(&Tok::Assign)?;
                let body = self.expr()?;
                Ok(Cmd::Def(n, ty, body))
            }
            Some(Tok::Axiom) => {
                let n = self.ident()?;
                self.expect(&Tok::Colon)?;
                let ty = self.expr()?;
                Ok(Cmd::Axiom(n, ty))
            }
            Some(Tok::Theorem) => {
                let n = self.ident()?;
                self.expect(&Tok::Colon)?;
                let ty = self.expr()?;
                self.expect(&Tok::Assign)?;
                let t = if self.eat(&Tok::By) {
                    self.tactic_block()?
                } else {
                    Tactic::Exact(self.expr()?)
                };
                Ok(Cmd::Theorem(n, ty, t))
            }
            Some(Tok::Check) => Ok(Cmd::Check(self.expr()?)),
            Some(Tok::Eval) => Ok(Cmd::Eval(self.expr()?)),
            Some(Tok::Print) => Ok(Cmd::Print(self.ident()?)),
            t => Err(KError::Parse(format!("expected command, got {t:?}"))),
        }
    }

    fn tactic_block(&mut self) -> KResult<Tactic> {
        let mut ts = vec![self.tactic()?];
        while self.eat(&Tok::Semi) {
            ts.push(self.tactic()?);
        }
        Ok(if ts.len() == 1 {
            ts.pop().unwrap()
        } else {
            Tactic::Seq(ts)
        })
    }

    fn tactic(&mut self) -> KResult<Tactic> {
        let name = self.ident()?;
        match name.as_str() {
            "intro" => Ok(Tactic::Intro(self.ident()?)),
            "exact" => Ok(Tactic::Exact(self.expr()?)),
            "apply" => Ok(Tactic::Apply(self.expr()?)),
            "refl" | "rfl" => Ok(Tactic::Refl),
            "assumption" => Ok(Tactic::Assumption),
            "sorry" => Ok(Tactic::Sorry),
            _ => Err(KError::Parse(format!("unknown tactic: {name}"))),
        }
    }

    pub fn expr(&mut self) -> KResult<Expr> {
        let head = self.binder_or_app()?;
        if self.eat(&Tok::Arrow) {
            let rhs = self.expr()?;
            Ok(Expr::Arrow(Box::new(head), Box::new(rhs)))
        } else {
            Ok(head)
        }
    }

    fn binder_or_app(&mut self) -> KResult<Expr> {
        match self.peek() {
            Some(Tok::Fun) => self.fun(),
            Some(Tok::Let) => self.let_(),
            Some(Tok::LParen) if self.starts_pi() => self.pi(),
            _ => self.app(),
        }
    }

    fn starts_pi(&self) -> bool {
        let save = self.pos;
        let mut p = save;
        if !matches!(self.toks.get(p), Some(Tok::LParen)) {
            return false;
        }
        p += 1;
        if !matches!(self.toks.get(p), Some(Tok::Ident(_))) {
            return false;
        }
        p += 1;
        matches!(self.toks.get(p), Some(Tok::Colon))
    }

    fn fun(&mut self) -> KResult<Expr> {
        self.bump();
        let mut binders = Vec::new();
        loop {
            if self.eat(&Tok::LParen) {
                let n = self.ident()?;
                self.expect(&Tok::Colon)?;
                let ty = self.expr()?;
                self.expect(&Tok::RParen)?;
                binders.push((n, Some(ty)));
            } else if let Some(Tok::Ident(_)) = self.peek() {
                let n = self.ident()?;
                binders.push((n, None));
            } else {
                break;
            }
            if matches!(self.peek(), Some(Tok::FatArrow)) {
                break;
            }
        }
        self.expect(&Tok::FatArrow)?;
        let body = self.expr()?;
        Ok(binders
            .into_iter()
            .rev()
            .fold(body, |b, (n, t)| Expr::Lam(n, t.map(Box::new), Box::new(b))))
    }

    fn let_(&mut self) -> KResult<Expr> {
        self.bump();
        let n = self.ident()?;
        let ty = if self.eat(&Tok::Colon) {
            Some(self.expr()?)
        } else {
            None
        };
        self.expect(&Tok::Assign)?;
        let val = self.expr()?;
        self.expect(&Tok::Semi)?;
        let body = self.expr()?;
        Ok(Expr::Let(
            n,
            ty.map(Box::new),
            Box::new(val),
            Box::new(body),
        ))
    }

    fn pi(&mut self) -> KResult<Expr> {
        self.expect(&Tok::LParen)?;
        let n = self.ident()?;
        self.expect(&Tok::Colon)?;
        let ty = self.expr()?;
        self.expect(&Tok::RParen)?;
        self.expect(&Tok::Arrow)?;
        let body = self.expr()?;
        Ok(Expr::Pi(n, Box::new(ty), Box::new(body)))
    }

    fn app(&mut self) -> KResult<Expr> {
        let mut head = self.atom()?;
        while matches!(
            self.peek(),
            Some(
                Tok::Ident(_)
                    | Tok::LParen
                    | Tok::Hole
                    | Tok::Nat(_)
                    | Tok::Prop
                    | Tok::TypeKw
                    | Tok::SortKw
            )
        ) {
            let arg = self.atom()?;
            head = Expr::App(Box::new(head), Box::new(arg));
        }
        Ok(head)
    }

    fn atom(&mut self) -> KResult<Expr> {
        match self.bump() {
            Some(Tok::Ident(n)) => Ok(Expr::Var(n)),
            Some(Tok::Hole) => Ok(Expr::Hole),
            Some(Tok::Prop) => Ok(Expr::Sort(Level::Zero)),
            Some(Tok::TypeKw) => {
                let n = if let Some(Tok::Nat(_)) = self.peek() {
                    match self.bump() {
                        Some(Tok::Nat(n)) => n + 1,
                        _ => unreachable!(),
                    }
                } else {
                    1
                };
                Ok(Expr::Sort(Level::nat(n)))
            }
            Some(Tok::SortKw) => match self.bump() {
                Some(Tok::Nat(n)) => Ok(Expr::Sort(Level::nat(n))),
                t => Err(KError::Parse(format!(
                    "expected level after Sort, got {t:?}"
                ))),
            },
            Some(Tok::LParen) => {
                let e = self.expr()?;
                self.expect(&Tok::RParen)?;
                Ok(e)
            }
            t => Err(KError::Parse(format!("expected atom, got {t:?}"))),
        }
    }
}

pub fn parse(src: &str) -> KResult<Vec<Cmd>> {
    Parser::new(src)?.cmds()
}
