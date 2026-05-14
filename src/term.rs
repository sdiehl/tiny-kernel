use std::fmt;

pub type Name = String;
pub type Idx = usize;
pub type Lvl = usize;
pub type MetaId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Zero,
    Succ(Box<Self>),
    Max(Box<Self>, Box<Self>),
    IMax(Box<Self>, Box<Self>),
}

impl Level {
    #[must_use]
    pub fn nat(n: u32) -> Self {
        (0..n).fold(Self::Zero, |l, _| Self::Succ(Box::new(l)))
    }

    #[must_use]
    pub fn succ(self) -> Self {
        Self::Succ(Box::new(self))
    }

    #[must_use]
    pub fn max(a: Self, b: Self) -> Self {
        match (&a, &b) {
            (Self::Zero, _) => b,
            (_, Self::Zero) => a,
            _ if a == b => a,
            _ => Self::Max(Box::new(a), Box::new(b)),
        }
    }

    #[must_use]
    pub fn imax(a: Self, b: Self) -> Self {
        match &b {
            Self::Zero => Self::Zero,
            Self::Succ(_) => Self::max(a, b),
            _ => Self::IMax(Box::new(a), Box::new(b)),
        }
    }

    #[must_use]
    pub fn to_nat(&self) -> Option<u32> {
        match self {
            Self::Zero => Some(0),
            Self::Succ(l) => l.to_nat().map(|n| n + 1),
            Self::Max(a, b) => Some(a.to_nat()?.max(b.to_nat()?)),
            Self::IMax(a, b) => match b.to_nat()? {
                0 => Some(0),
                n => Some(a.to_nat()?.max(n)),
            },
        }
    }

    #[must_use]
    pub fn leq(&self, other: &Self) -> bool {
        match (self.to_nat(), other.to_nat()) {
            (Some(a), Some(b)) => a <= b,
            _ => self == other,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_nat() {
            Some(n) => write!(f, "{n}"),
            None => match self {
                Self::Zero => write!(f, "0"),
                Self::Succ(l) => write!(f, "({l})+1"),
                Self::Max(a, b) => write!(f, "max({a},{b})"),
                Self::IMax(a, b) => write!(f, "imax({a},{b})"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Term {
    Var(Idx),
    Sort(Level),
    App(Box<Self>, Box<Self>),
    Lam(Name, Box<Self>, Box<Self>),
    Pi(Name, Box<Self>, Box<Self>),
    Let(Name, Box<Self>, Box<Self>, Box<Self>),
    Const(Name),
    Meta(MetaId),
}

impl Term {
    #[must_use]
    pub fn app(self, arg: Self) -> Self {
        Self::App(Box::new(self), Box::new(arg))
    }

    #[must_use]
    pub fn apps<I: IntoIterator<Item = Self>>(self, args: I) -> Self {
        args.into_iter().fold(self, Self::app)
    }

    #[must_use]
    pub fn pi(x: &str, dom: Self, body: Self) -> Self {
        Self::Pi(x.into(), Box::new(dom), Box::new(body))
    }

    #[must_use]
    pub fn lam(x: &str, ty: Self, body: Self) -> Self {
        Self::Lam(x.into(), Box::new(ty), Box::new(body))
    }

    #[must_use]
    pub fn arrow(a: Self, b: Self) -> Self {
        Self::Pi("_".into(), Box::new(a), Box::new(b))
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        pretty(self, &mut Vec::new(), 0, f)
    }
}

fn pretty(t: &Term, ns: &mut Vec<Name>, min_prec: u8, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use Term::{App, Const, Lam, Let, Meta, Pi, Sort, Var};
    let my_prec = match t {
        Var(_) | Const(_) | Meta(_) | Sort(_) => 100,
        App(_, _) => 50,
        Pi(x, _, _) if x == "_" => 30,
        Pi(_, _, _) | Lam(_, _, _) | Let(_, _, _, _) => 10,
    };
    let wrap = my_prec < min_prec;
    if wrap {
        write!(f, "(")?;
    }
    match t {
        Var(i) => match ns.len().checked_sub(i + 1).and_then(|j| ns.get(j)) {
            Some(n) if !n.is_empty() && n != "_" => write!(f, "{n}")?,
            _ => write!(f, "#{i}")?,
        },
        Sort(l) => match l.to_nat() {
            Some(0) => write!(f, "Prop")?,
            Some(1) => write!(f, "Type")?,
            Some(n) => write!(f, "Type {}", n - 1)?,
            None => write!(f, "Sort {l}")?,
        },
        Const(n) => write!(f, "{n}")?,
        Meta(m) => write!(f, "?{m}")?,
        App(g, a) => {
            pretty(g, ns, 50, f)?;
            write!(f, " ")?;
            pretty(a, ns, 51, f)?;
        }
        Lam(x, ty, body) => {
            write!(f, "fun ({x} : ")?;
            pretty(ty, ns, 0, f)?;
            write!(f, ") => ")?;
            ns.push(x.clone());
            pretty(body, ns, 10, f)?;
            ns.pop();
        }
        Pi(x, ty, body) if x == "_" => {
            pretty(ty, ns, 51, f)?;
            write!(f, " -> ")?;
            ns.push(x.clone());
            pretty(body, ns, 30, f)?;
            ns.pop();
        }
        Pi(x, ty, body) => {
            write!(f, "({x} : ")?;
            pretty(ty, ns, 0, f)?;
            write!(f, ") -> ")?;
            ns.push(x.clone());
            pretty(body, ns, 10, f)?;
            ns.pop();
        }
        Let(x, ty, val, body) => {
            write!(f, "let {x} : ")?;
            pretty(ty, ns, 0, f)?;
            write!(f, " := ")?;
            pretty(val, ns, 0, f)?;
            write!(f, "; ")?;
            ns.push(x.clone());
            pretty(body, ns, 10, f)?;
            ns.pop();
        }
    }
    if wrap {
        write!(f, ")")?;
    }
    Ok(())
}
