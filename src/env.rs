use std::collections::HashMap;

use crate::term::{MetaId, Name};
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Decl {
    pub ty: Value,
    pub body: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub ty: Value,
    pub solution: Option<Value>,
}

#[derive(Debug, Default)]
pub struct Globals {
    pub decls: HashMap<Name, Decl>,
    pub metas: Vec<Meta>,
}

impl Globals {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_decl(&mut self, n: Name, ty: Value, body: Option<Value>) {
        self.decls.insert(n, Decl { ty, body });
    }

    #[must_use]
    pub fn lookup(&self, n: &str) -> Option<&Decl> {
        self.decls.get(n)
    }

    pub fn fresh_meta(&mut self, ty: Value) -> MetaId {
        let m = self.metas.len();
        self.metas.push(Meta { ty, solution: None });
        m
    }

    #[must_use]
    pub fn meta(&self, m: MetaId) -> &Meta {
        &self.metas[m]
    }

    pub fn solve(&mut self, m: MetaId, v: Value) {
        self.metas[m].solution = Some(v);
    }

    pub fn unsolved(&self) -> impl Iterator<Item = MetaId> + '_ {
        self.metas
            .iter()
            .enumerate()
            .filter_map(|(i, m)| m.solution.is_none().then_some(i))
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalCtx {
    pub names: Vec<Name>,
    pub types: Vec<Value>,
    pub env: Vec<Value>,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone)]
pub enum Binding {
    Bound,
    Defined(Value),
}

impl LocalCtx {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn bind(&self, name: Name, ty: Value) -> Self {
        let mut c = self.clone();
        let lvl = c.env.len();
        c.env.push(Value::var(lvl));
        c.types.push(ty);
        c.names.push(name);
        c.bindings.push(Binding::Bound);
        c
    }

    #[must_use]
    pub fn define(&self, name: Name, ty: Value, val: Value) -> Self {
        let mut c = self.clone();
        c.env.push(val.clone());
        c.types.push(ty);
        c.names.push(name);
        c.bindings.push(Binding::Defined(val));
        c
    }

    #[must_use]
    pub const fn lvl(&self) -> usize {
        self.env.len()
    }

    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<(usize, &Value)> {
        for (i, n) in self.names.iter().enumerate().rev() {
            if n == name {
                let idx = self.names.len() - 1 - i;
                return Some((idx, &self.types[i]));
            }
        }
        None
    }
}
