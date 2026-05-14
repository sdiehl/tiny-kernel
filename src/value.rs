use std::rc::Rc;

use crate::term::{Level, Lvl, MetaId, Name, Term};

pub type Env = Vec<Value>;

#[derive(Debug, Clone)]
pub struct Closure {
    pub env: Env,
    pub body: Term,
}

#[derive(Debug, Clone)]
pub enum Value {
    VLam(Name, Rc<Self>, Closure),
    VPi(Name, Rc<Self>, Closure),
    VSort(Level),
    VRigid(Lvl, Vec<Self>),
    VFlex(MetaId, Vec<Self>),
    VConst(Name, Vec<Self>),
}

impl Value {
    #[must_use]
    pub const fn var(l: Lvl) -> Self {
        Self::VRigid(l, vec![])
    }

    #[must_use]
    pub const fn meta(m: MetaId) -> Self {
        Self::VFlex(m, vec![])
    }

    pub fn const_(n: impl Into<Name>) -> Self {
        Self::VConst(n.into(), vec![])
    }
}
