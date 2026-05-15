use crate::elab::{check, infer, zonk};
use crate::env::{Globals, LocalCtx};
use crate::errors::{KError, KResult};
use crate::eval::{eval, force, quote, whnf};
use crate::parse::parse;
use crate::surface::Cmd;
use crate::tactic::{run, TacticState};
use crate::term::{Level, Term};
use crate::value::Value;

pub const PRELUDE: &str = include_str!("prelude.tk");

pub fn prelude(g: &mut Globals) -> KResult<()> {
    let prop = Value::VSort(Level::Zero);
    let ty = Value::VSort(Level::nat(1));
    g.add_decl("Prop".into(), Value::VSort(Level::nat(1)), Some(prop));
    g.add_decl("Type".into(), Value::VSort(Level::nat(2)), Some(ty));

    for c in parse(PRELUDE).map_err(|e| KError::Prelude(Box::new(e)))? {
        run_cmd(g, &c).map_err(|e| KError::Prelude(Box::new(e)))?;
    }
    Ok(())
}

pub fn run_program(src: &str) -> KResult<Vec<String>> {
    let mut g = Globals::new();
    prelude(&mut g)?;
    let mut out = Vec::new();
    for c in parse(src)? {
        if let Some(line) = run_cmd(&mut g, &c)? {
            out.push(line);
        }
    }
    Ok(out)
}

pub fn run_cmd(g: &mut Globals, c: &Cmd) -> KResult<Option<String>> {
    match c {
        Cmd::Axiom(n, ty) => {
            let ctx = LocalCtx::new();
            let (ty_t, ty_sort) = infer(g, &ctx, ty)?;
            ensure_sort(g, &ty_sort)?;
            let ty_v = eval(g, &ctx.env, &ty_t);
            g.add_decl(n.clone(), ty_v, None);
            Ok(Some(format!("axiom {n} : {ty_t}")))
        }
        Cmd::Def(n, ann, body) => {
            let ctx = LocalCtx::new();
            let (body_t, ty_v) = match ann {
                Some(a) => {
                    let (a_t, _) = infer(g, &ctx, a)?;
                    let a_v = eval(g, &ctx.env, &a_t);
                    let b = check(g, &ctx, body, &a_v)?;
                    (b, a_v)
                }
                None => infer(g, &ctx, body)?,
            };
            let body_t = zonk(g, 0, &body_t);
            let body_v = eval(g, &ctx.env, &body_t);
            g.add_decl(n.clone(), ty_v.clone(), Some(body_v));
            Ok(Some(format!(
                "def {n} : {} := {body_t}",
                quote(g, 0, &ty_v)
            )))
        }
        Cmd::Theorem(n, ty, tac) => {
            let ctx = LocalCtx::new();
            let (ty_t, _) = infer(g, &ctx, ty)?;
            let ty_v = eval(g, &ctx.env, &ty_t);
            let goal_meta = g.fresh_meta(ty_v.clone());
            let mut st = TacticState::new(ctx.clone(), ty_v.clone(), goal_meta);
            run(g, &mut st, tac)?;
            if !st.goals.is_empty() {
                return Err(KError::Tactic(format!("{} goals remain", st.goals.len())));
            }
            let proof_t = zonk(g, 0, &Term::Meta(goal_meta));
            let proof_v = eval(g, &ctx.env, &proof_t);
            g.add_decl(n.clone(), ty_v, Some(proof_v));
            Ok(Some(format!("theorem {n} : {ty_t} := {proof_t}")))
        }
        Cmd::Check(e) => {
            let ctx = LocalCtx::new();
            let (t, ty) = infer(g, &ctx, e)?;
            let t = zonk(g, 0, &t);
            let ty_t = quote(g, 0, &ty);
            Ok(Some(format!("{t} : {ty_t}")))
        }
        Cmd::Eval(e) => {
            let ctx = LocalCtx::new();
            let (t, _) = infer(g, &ctx, e)?;
            let v = eval(g, &ctx.env, &t);
            let v2 = whnf(g, v);
            let t2 = quote(g, 0, &v2);
            Ok(Some(format!("{t2}")))
        }
        Cmd::Print(n) => {
            let d = g
                .lookup(n)
                .ok_or_else(|| KError::Unbound(n.clone()))?
                .clone();
            let ty_t = quote(g, 0, &d.ty);
            let body = d
                .body
                .as_ref()
                .map(|v| format!(" := {}", quote(g, 0, v)))
                .unwrap_or_default();
            Ok(Some(format!("{n} : {ty_t}{body}")))
        }
    }
}

fn ensure_sort(g: &Globals, v: &Value) -> KResult<Level> {
    match force(g, v.clone()) {
        Value::VSort(l) => Ok(l),
        other => Err(KError::NotType(quote(g, 0, &other))),
    }
}
