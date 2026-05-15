use std::collections::HashMap;

use crate::env::Globals;
use crate::errors::{KError, KResult};
use crate::eval::{apply, conv, eval, force, open, quote};
use crate::term::{Level, Lvl, MetaId, Term};
use crate::value::Value;

pub fn unify(g: &mut Globals, lvl: Lvl, types: &[Value], a: &Value, b: &Value) -> KResult<()> {
    let a = force(g, a.clone());
    let b = force(g, b.clone());
    if conv(g, lvl, &a, &b) {
        return Ok(());
    }
    match (a, b) {
        (Value::VPi(_, t1, c1), Value::VPi(_, t2, c2)) => {
            unify(g, lvl, types, &t1, &t2)?;
            let mut types2 = types.to_vec();
            types2.push((*t1).clone());
            unify(
                g,
                lvl + 1,
                &types2,
                &open(&c1, Value::var(lvl), g),
                &open(&c2, Value::var(lvl), g),
            )
        }
        (Value::VLam(_, _, c1), Value::VLam(_, t2, c2)) => {
            let mut types2 = types.to_vec();
            types2.push((*t2).clone());
            unify(
                g,
                lvl + 1,
                &types2,
                &open(&c1, Value::var(lvl), g),
                &open(&c2, Value::var(lvl), g),
            )
        }
        (Value::VLam(_, t, c), v) | (v, Value::VLam(_, t, c)) => {
            let mut types2 = types.to_vec();
            types2.push((*t).clone());
            unify(
                g,
                lvl + 1,
                &types2,
                &open(&c, Value::var(lvl), g),
                &apply(g, v, Value::var(lvl)),
            )
        }
        (Value::VRigid(l1, sp1), Value::VRigid(l2, sp2)) if l1 == l2 && sp1.len() == sp2.len() => {
            for (x, y) in sp1.iter().zip(sp2.iter()) {
                unify(g, lvl, types, x, y)?;
            }
            Ok(())
        }
        (Value::VConst(n1, sp1), Value::VConst(n2, sp2)) if n1 == n2 && sp1.len() == sp2.len() => {
            for (x, y) in sp1.iter().zip(sp2.iter()) {
                unify(g, lvl, types, x, y)?;
            }
            Ok(())
        }
        (Value::VFlex(m1, sp1), Value::VFlex(m2, sp2)) if m1 == m2 && sp1.len() == sp2.len() => {
            for (x, y) in sp1.iter().zip(sp2.iter()) {
                unify(g, lvl, types, x, y)?;
            }
            Ok(())
        }
        (Value::VFlex(m, sp), rhs) | (rhs, Value::VFlex(m, sp)) => {
            solve(g, lvl, types, m, &sp, &rhs)
        }
        // All call sites in `elab.rs` invoke unify as `unify(.., actual, expected)`.
        (a, b) => Err(KError::TypeMismatch {
            expected: quote(g, lvl, &b),
            actual: quote(g, lvl, &a),
        }),
    }
}

struct PRen {
    dom: Lvl,
    cod: Lvl,
    map: HashMap<Lvl, Lvl>,
}

impl PRen {
    fn lift(&self) -> Self {
        let mut map = self.map.clone();
        map.insert(self.cod, self.dom);
        Self {
            dom: self.dom + 1,
            cod: self.cod + 1,
            map,
        }
    }
}

fn invert(g: &Globals, gamma: Lvl, sp: &[Value]) -> KResult<PRen> {
    let mut map = HashMap::new();
    let mut dom: Lvl = 0;
    for v in sp {
        match force(g, v.clone()) {
            Value::VRigid(l, s) if s.is_empty() => {
                if map.contains_key(&l) {
                    return Err(KError::NonPattern("repeated var in spine".into()));
                }
                map.insert(l, dom);
                dom += 1;
            }
            _ => return Err(KError::NonPattern("non-variable in spine".into())),
        }
    }
    Ok(PRen {
        dom,
        cod: gamma,
        map,
    })
}

fn rename(g: &Globals, m: MetaId, pren: &PRen, v: &Value) -> KResult<Term> {
    match force(g, v.clone()) {
        Value::VSort(l) => Ok(Term::Sort(l)),
        Value::VRigid(l, sp) => pren.map.get(&l).map_or_else(
            || Err(KError::Escape(l)),
            |d| {
                let idx = pren.dom - 1 - d;
                rename_spine(g, m, pren, Term::Var(idx), &sp)
            },
        ),
        Value::VFlex(m2, sp) => {
            if m2 == m {
                return Err(KError::Occurs(m));
            }
            rename_spine(g, m, pren, Term::Meta(m2), &sp)
        }
        Value::VConst(n, sp) => rename_spine(g, m, pren, Term::Const(n), &sp),
        Value::VLam(x, ty, cl) => {
            let ty_t = rename(g, m, pren, &ty)?;
            let body = open(&cl, Value::var(pren.cod), g);
            let body_t = rename(g, m, &pren.lift(), &body)?;
            Ok(Term::Lam(x, Box::new(ty_t), Box::new(body_t)))
        }
        Value::VPi(x, ty, cl) => {
            let ty_t = rename(g, m, pren, &ty)?;
            let body = open(&cl, Value::var(pren.cod), g);
            let body_t = rename(g, m, &pren.lift(), &body)?;
            Ok(Term::Pi(x, Box::new(ty_t), Box::new(body_t)))
        }
    }
}

fn rename_spine(g: &Globals, m: MetaId, pren: &PRen, head: Term, sp: &[Value]) -> KResult<Term> {
    let mut t = head;
    for a in sp {
        t = t.app(rename(g, m, pren, a)?);
    }
    Ok(t)
}

fn solve(
    g: &mut Globals,
    gamma: Lvl,
    types: &[Value],
    m: MetaId,
    sp: &[Value],
    rhs: &Value,
) -> KResult<()> {
    let pren = invert(g, gamma, sp)?;
    let body = rename(g, m, &pren, rhs)?;
    let sol_term = wrap_lambdas(types, sp, body, g, gamma);
    let sol = eval(g, &Vec::new(), &sol_term);
    g.solve(m, sol);
    Ok(())
}

fn wrap_lambdas(types: &[Value], sp: &[Value], body: Term, g: &Globals, gamma: Lvl) -> Term {
    let mut t = body;
    for v in sp.iter().rev() {
        let Value::VRigid(lvl, _) = force(g, v.clone()) else {
            unreachable!()
        };
        let ty_v = types.get(lvl).cloned().unwrap_or(Value::VSort(Level::Zero));
        let ty_t = quote(g, gamma, &ty_v);
        t = Term::Lam(format!("x{lvl}"), Box::new(ty_t), Box::new(t));
    }
    t
}
