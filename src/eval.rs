use std::rc::Rc;

use crate::env::Globals;
use crate::term::{Idx, Lvl, Term};
use crate::value::{Closure, Env, Value};

#[must_use]
pub fn eval(g: &Globals, env: &Env, t: &Term) -> Value {
    use Term::{App, Const, Lam, Let, Meta, Pi, Sort, Var};
    match t {
        Var(i) => env[env.len() - 1 - i].clone(),
        Sort(l) => Value::VSort(l.clone()),
        App(f, a) => {
            let fv = eval(g, env, f);
            let av = eval(g, env, a);
            apply(g, fv, av)
        }
        Lam(x, ty, body) => Value::VLam(
            x.clone(),
            Rc::new(eval(g, env, ty)),
            Closure {
                env: env.clone(),
                body: (**body).clone(),
            },
        ),
        Pi(x, ty, body) => Value::VPi(
            x.clone(),
            Rc::new(eval(g, env, ty)),
            Closure {
                env: env.clone(),
                body: (**body).clone(),
            },
        ),
        Let(_, _, val, body) => {
            let v = eval(g, env, val);
            let mut env2 = env.clone();
            env2.push(v);
            eval(g, &env2, body)
        }
        Const(n) => g
            .lookup(n)
            .and_then(|d| d.body.clone())
            .unwrap_or_else(|| Value::const_(n.clone())),
        Meta(m) => g
            .meta(*m)
            .solution
            .clone()
            .unwrap_or_else(|| Value::meta(*m)),
    }
}

#[must_use]
pub fn apply(g: &Globals, f: Value, a: Value) -> Value {
    match f {
        Value::VLam(_, _, cl) => open(&cl, a, g),
        Value::VRigid(l, mut sp) => {
            sp.push(a);
            Value::VRigid(l, sp)
        }
        Value::VFlex(m, mut sp) => {
            sp.push(a);
            match g.meta(m).solution.clone() {
                Some(sol) => sp.into_iter().fold(sol, |acc, x| apply(g, acc, x)),
                None => Value::VFlex(m, sp),
            }
        }
        Value::VConst(n, mut sp) => {
            sp.push(a);
            Value::VConst(n, sp)
        }
        _ => panic!("apply: head is not a function: {f:?}"),
    }
}

#[must_use]
pub fn open(cl: &Closure, v: Value, g: &Globals) -> Value {
    let mut env = cl.env.clone();
    env.push(v);
    eval(g, &env, &cl.body)
}

#[must_use]
pub fn quote(g: &Globals, lvl: Lvl, v: &Value) -> Term {
    match force(g, v.clone()) {
        Value::VSort(l) => Term::Sort(l),
        Value::VLam(x, ty, cl) => {
            let body = open(&cl, Value::var(lvl), g);
            Term::Lam(
                x,
                Box::new(quote(g, lvl, &ty)),
                Box::new(quote(g, lvl + 1, &body)),
            )
        }
        Value::VPi(x, ty, cl) => {
            let body = open(&cl, Value::var(lvl), g);
            Term::Pi(
                x,
                Box::new(quote(g, lvl, &ty)),
                Box::new(quote(g, lvl + 1, &body)),
            )
        }
        Value::VRigid(l, sp) => sp.iter().fold(Term::Var(lvl_to_idx(lvl, l)), |t, a| {
            t.app(quote(g, lvl, a))
        }),
        Value::VFlex(m, sp) => sp
            .iter()
            .fold(Term::Meta(m), |t, a| t.app(quote(g, lvl, a))),
        Value::VConst(n, sp) => sp
            .iter()
            .fold(Term::Const(n), |t, a| t.app(quote(g, lvl, a))),
    }
}

const fn lvl_to_idx(env_lvl: Lvl, var_lvl: Lvl) -> Idx {
    env_lvl - var_lvl - 1
}

#[must_use]
pub fn force(g: &Globals, v: Value) -> Value {
    match v {
        Value::VFlex(m, sp) => match g.meta(m).solution.clone() {
            Some(sol) => {
                let v2 = sp.into_iter().fold(sol, |acc, x| apply(g, acc, x));
                force(g, v2)
            }
            None => Value::VFlex(m, sp),
        },
        v => v,
    }
}

#[must_use]
pub fn whnf(g: &Globals, v: Value) -> Value {
    match force(g, v) {
        Value::VConst(n, sp) => match g.lookup(&n).and_then(|d| d.body.clone()) {
            Some(b) => {
                let r = sp.into_iter().fold(b, |acc, x| apply(g, acc, x));
                whnf(g, r)
            }
            None => Value::VConst(n, sp),
        },
        v => v,
    }
}

#[must_use]
pub fn conv(g: &Globals, lvl: Lvl, a: &Value, b: &Value) -> bool {
    let a = force(g, a.clone());
    let b = force(g, b.clone());
    if conv_no_unfold(g, lvl, &a, &b) {
        return true;
    }
    let a2 = whnf(g, a);
    let b2 = whnf(g, b);
    conv_no_unfold(g, lvl, &a2, &b2)
}

fn conv_no_unfold(g: &Globals, lvl: Lvl, a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::VSort(u), Value::VSort(v)) => u == v || u.to_nat() == v.to_nat(),
        (Value::VRigid(l1, sp1), Value::VRigid(l2, sp2)) => {
            l1 == l2 && conv_spine(g, lvl, sp1, sp2)
        }
        (Value::VFlex(m1, sp1), Value::VFlex(m2, sp2)) if m1 == m2 => conv_spine(g, lvl, sp1, sp2),
        (Value::VConst(n1, sp1), Value::VConst(n2, sp2)) if n1 == n2 => {
            conv_spine(g, lvl, sp1, sp2)
        }
        (Value::VPi(_, t1, c1), Value::VPi(_, t2, c2)) => {
            conv(g, lvl, t1, t2)
                && conv(
                    g,
                    lvl + 1,
                    &open(c1, Value::var(lvl), g),
                    &open(c2, Value::var(lvl), g),
                )
        }
        (Value::VLam(_, _, c1), Value::VLam(_, _, c2)) => conv(
            g,
            lvl + 1,
            &open(c1, Value::var(lvl), g),
            &open(c2, Value::var(lvl), g),
        ),
        (Value::VLam(_, _, c), v) | (v, Value::VLam(_, _, c)) => {
            let body = open(c, Value::var(lvl), g);
            let applied = apply(g, v.clone(), Value::var(lvl));
            conv(g, lvl + 1, &body, &applied)
        }
        _ => false,
    }
}

fn conv_spine(g: &Globals, lvl: Lvl, a: &[Value], b: &[Value]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| conv(g, lvl, x, y))
}
