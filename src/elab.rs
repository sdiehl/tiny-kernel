use std::rc::Rc;

use crate::env::{Globals, LocalCtx};
use crate::errors::{KError, KResult};
use crate::eval::{eval, force, open, quote};
use crate::surface::Expr;
use crate::term::{Level, Term};
use crate::unify::unify;
use crate::value::{Closure, Value};

pub fn fresh_meta_term(g: &mut Globals, ctx: &LocalCtx, ty: Value) -> Term {
    let m = g.fresh_meta(ty);
    let mut t = Term::Meta(m);
    for i in 0..ctx.lvl() {
        t = t.app(Term::Var(ctx.lvl() - 1 - i));
    }
    t
}

pub fn fresh_type_meta(g: &mut Globals, ctx: &LocalCtx) -> (Term, Value) {
    let sort = Value::VSort(Level::nat(1));
    let t = fresh_meta_term(g, ctx, sort);
    let v = eval(g, &ctx.env, &t);
    (t, v)
}

pub fn check(g: &mut Globals, ctx: &LocalCtx, e: &Expr, ty: &Value) -> KResult<Term> {
    let ty_forced = force(g, ty.clone());
    match (e, &ty_forced) {
        (Expr::Lam(x, ann, body), Value::VPi(_, dom, cod)) => {
            if let Some(a) = ann {
                let (a_t, a_sort) = infer(g, ctx, a)?;
                expect_sort(g, ctx, &a_sort)?;
                let a_v = eval(g, &ctx.env, &a_t);
                unify(g, ctx.lvl(), &ctx.types, &a_v, dom)?;
            }
            let ctx2 = ctx.bind(x.clone(), (**dom).clone());
            let body_ty = open(cod, Value::var(ctx.lvl()), g);
            let body_t = check(g, &ctx2, body, &body_ty)?;
            let dom_t = quote(g, ctx.lvl(), dom);
            Ok(Term::Lam(x.clone(), Box::new(dom_t), Box::new(body_t)))
        }
        (Expr::Let(x, ann, val, body), _) => {
            let (val_t, val_ty_v) = match ann {
                Some(a) => {
                    let a_t = check(g, ctx, a, &Value::VSort(Level::nat(1)))?;
                    let a_v = eval(g, &ctx.env, &a_t);
                    let v_t = check(g, ctx, val, &a_v)?;
                    (v_t, a_v)
                }
                None => infer(g, ctx, val)?,
            };
            let val_v = eval(g, &ctx.env, &val_t);
            let ctx2 = ctx.define(x.clone(), val_ty_v.clone(), val_v);
            let body_t = check(g, &ctx2, body, ty)?;
            let ty_t = quote(g, ctx.lvl(), &val_ty_v);
            Ok(Term::Let(
                x.clone(),
                Box::new(ty_t),
                Box::new(val_t),
                Box::new(body_t),
            ))
        }
        (Expr::Hole, _) => Ok(fresh_meta_term(g, ctx, ty.clone())),
        _ => {
            let (t, inferred) = infer(g, ctx, e)?;
            unify(g, ctx.lvl(), &ctx.types, &inferred, ty)?;
            Ok(t)
        }
    }
}

pub fn infer(g: &mut Globals, ctx: &LocalCtx, e: &Expr) -> KResult<(Term, Value)> {
    match e {
        Expr::Var(n) => {
            if let Some((i, ty)) = ctx.lookup(n) {
                return Ok((Term::Var(i), ty.clone()));
            }
            g.lookup(n).map_or_else(
                || Err(KError::Unbound(n.clone())),
                |d| Ok((Term::Const(n.clone()), d.ty.clone())),
            )
        }
        Expr::Sort(l) => Ok((Term::Sort(l.clone()), Value::VSort(l.clone().succ()))),
        Expr::Hole => {
            let (_, ty_v) = fresh_type_meta(g, ctx);
            let t = fresh_meta_term(g, ctx, ty_v.clone());
            Ok((t, ty_v))
        }
        Expr::App(f, a) => {
            let (f_t, f_ty) = infer(g, ctx, f)?;
            let f_ty = force(g, f_ty);
            let (dom_v, cod_cl) = match f_ty {
                Value::VPi(_, dom, cod) => (dom, cod),
                Value::VFlex(_, _) => {
                    let (_, dom_v) = fresh_type_meta(g, ctx);
                    let ctx2 = ctx.bind("x".into(), dom_v.clone());
                    let (_, cod_v) = fresh_type_meta(g, &ctx2);
                    let cod_t = quote(g, ctx2.lvl(), &cod_v);
                    let pi = Value::VPi(
                        "x".into(),
                        Rc::new(dom_v),
                        Closure {
                            env: ctx.env.clone(),
                            body: cod_t,
                        },
                    );
                    unify(g, ctx.lvl(), &ctx.types, &f_ty, &pi)?;
                    match pi {
                        Value::VPi(_, d, c) => (d, c),
                        _ => unreachable!(),
                    }
                }
                other => return Err(KError::NotFn(quote(g, ctx.lvl(), &other))),
            };
            let a_t = check(g, ctx, a, &dom_v)?;
            let a_v = eval(g, &ctx.env, &a_t);
            let result_ty = open(&cod_cl, a_v, g);
            Ok((f_t.app(a_t), result_ty))
        }
        Expr::Lam(x, ann, body) => {
            let (dom_t, dom_v) = if let Some(a) = ann {
                let (t, a_sort) = infer(g, ctx, a)?;
                expect_sort(g, ctx, &a_sort)?;
                let v = eval(g, &ctx.env, &t);
                (t, v)
            } else {
                let (_, v) = fresh_type_meta(g, ctx);
                let t = quote(g, ctx.lvl(), &v);
                (t, v)
            };
            let ctx2 = ctx.bind(x.clone(), dom_v.clone());
            let (body_t, body_ty) = infer(g, &ctx2, body)?;
            let body_ty_t = quote(g, ctx2.lvl(), &body_ty);
            let pi = Value::VPi(
                x.clone(),
                Rc::new(dom_v),
                Closure {
                    env: ctx.env.clone(),
                    body: body_ty_t,
                },
            );
            Ok((Term::Lam(x.clone(), Box::new(dom_t), Box::new(body_t)), pi))
        }
        Expr::Pi(x, dom, body) => check_pi(g, ctx, x, dom, body),
        Expr::Arrow(dom, body) => check_pi(g, ctx, "_", dom, body),
        Expr::Let(x, ann, val, body) => {
            let (val_t, val_ty) = match ann {
                Some(a) => {
                    let a_t = check(g, ctx, a, &Value::VSort(Level::nat(1)))?;
                    let a_v = eval(g, &ctx.env, &a_t);
                    let v_t = check(g, ctx, val, &a_v)?;
                    (v_t, a_v)
                }
                None => infer(g, ctx, val)?,
            };
            let val_v = eval(g, &ctx.env, &val_t);
            let ctx2 = ctx.define(x.clone(), val_ty.clone(), val_v);
            let (body_t, body_ty) = infer(g, &ctx2, body)?;
            let ty_t = quote(g, ctx.lvl(), &val_ty);
            Ok((
                Term::Let(x.clone(), Box::new(ty_t), Box::new(val_t), Box::new(body_t)),
                body_ty,
            ))
        }
    }
}

fn check_pi(
    g: &mut Globals,
    ctx: &LocalCtx,
    x: &str,
    dom: &Expr,
    body: &Expr,
) -> KResult<(Term, Value)> {
    let (dom_t, dom_sort) = infer(g, ctx, dom)?;
    let dom_lvl = expect_sort(g, ctx, &dom_sort)?;
    let dom_v = eval(g, &ctx.env, &dom_t);
    let ctx2 = ctx.bind(x.into(), dom_v);
    let (body_t, body_sort) = infer(g, &ctx2, body)?;
    let body_lvl = expect_sort(g, &ctx2, &body_sort)?;
    let result = Level::imax(dom_lvl, body_lvl);
    Ok((
        Term::Pi(x.into(), Box::new(dom_t), Box::new(body_t)),
        Value::VSort(result),
    ))
}

fn expect_sort(g: &mut Globals, ctx: &LocalCtx, v: &Value) -> KResult<Level> {
    match force(g, v.clone()) {
        Value::VSort(l) => Ok(l),
        Value::VFlex(_, _) => {
            let l = Level::nat(1);
            let sort = Value::VSort(l.clone());
            unify(g, ctx.lvl(), &ctx.types, v, &sort)?;
            Ok(l)
        }
        other => Err(KError::NotType(quote(g, ctx.lvl(), &other))),
    }
}

#[must_use]
pub fn zonk(g: &Globals, ctx_lvl: usize, t: &Term) -> Term {
    let v = eval(g, &Vec::new(), t);
    quote(g, ctx_lvl, &v)
}
