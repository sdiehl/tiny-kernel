use crate::elab::{check, infer};
use crate::env::{Globals, LocalCtx};
use crate::errors::{KError, KResult};
use crate::eval::{conv, eval, force, open, quote};
use crate::surface::{Expr, Tactic};
use crate::term::{MetaId, Term};
use crate::unify::unify;
use crate::value::Value;

#[derive(Debug)]
pub struct Goal {
    pub ctx: LocalCtx,
    pub ty: Value,
    pub meta: MetaId,
}

#[derive(Debug, Default)]
pub struct TacticState {
    pub goals: Vec<Goal>,
}

impl TacticState {
    #[must_use]
    pub fn new(ctx: LocalCtx, ty: Value, meta: MetaId) -> Self {
        Self {
            goals: vec![Goal { ctx, ty, meta }],
        }
    }
}

pub fn run(g: &mut Globals, st: &mut TacticState, t: &Tactic) -> KResult<()> {
    match t {
        Tactic::Seq(ts) => {
            for t in ts {
                run(g, st, t)?;
            }
            Ok(())
        }
        Tactic::Intro(x) => intro(g, st, x),
        Tactic::Exact(e) => exact(g, st, e),
        Tactic::Apply(e) => apply_tac(g, st, e),
        Tactic::Refl => refl(g, st),
        Tactic::Assumption => assumption(g, st),
        Tactic::Sorry => {
            sorry(g, st);
            Ok(())
        }
    }
}

fn close_goal(g: &mut Globals, goal: &Goal, proof: Term) {
    let mut t = proof;
    for i in (0..goal.ctx.lvl()).rev() {
        let ty_t = quote(g, i, &goal.ctx.types[i]);
        let name = goal.ctx.names[i].clone();
        t = Term::Lam(name, Box::new(ty_t), Box::new(t));
    }
    let v = eval(g, &Vec::new(), &t);
    g.solve(goal.meta, v);
}

fn meta_app_in(m: MetaId, ctx: &LocalCtx) -> Term {
    let mut t = Term::Meta(m);
    for i in 0..ctx.lvl() {
        t = t.app(Term::Var(ctx.lvl() - 1 - i));
    }
    t
}

fn intro(g: &mut Globals, st: &mut TacticState, x: &str) -> KResult<()> {
    let goal = st.goals.remove(0);
    let ty = force(g, goal.ty.clone());
    match ty {
        Value::VPi(_, dom, cod) => {
            let ctx2 = goal.ctx.bind(x.into(), (*dom).clone());
            let body_ty = open(&cod, Value::var(goal.ctx.lvl()), g);
            let new_meta = g.fresh_meta(body_ty.clone());
            let body_proof = meta_app_in(new_meta, &ctx2);
            let dom_t = quote(g, goal.ctx.lvl(), &dom);
            let lam = Term::Lam(x.into(), Box::new(dom_t), Box::new(body_proof));
            close_goal(g, &goal, lam);
            st.goals.insert(
                0,
                Goal {
                    ctx: ctx2,
                    ty: body_ty,
                    meta: new_meta,
                },
            );
            Ok(())
        }
        other => Err(KError::Tactic(format!(
            "intro: goal is not a Pi: {}",
            quote(g, goal.ctx.lvl(), &other)
        ))),
    }
}

fn exact(g: &mut Globals, st: &mut TacticState, e: &Expr) -> KResult<()> {
    let goal = st.goals.remove(0);
    let proof_t = check(g, &goal.ctx, e, &goal.ty)?;
    close_goal(g, &goal, proof_t);
    Ok(())
}

fn apply_tac(g: &mut Globals, st: &mut TacticState, e: &Expr) -> KResult<()> {
    let goal = st.goals.remove(0);
    let (head_t, head_ty) = infer(g, &goal.ctx, e)?;
    let mut proof_t = head_t;
    let mut ty = head_ty;
    let mut new_metas = Vec::new();
    loop {
        let forced = force(g, ty.clone());
        match forced {
            Value::VPi(_, dom, cod) => {
                let m = g.fresh_meta((*dom).clone());
                let arg_t = meta_app_in(m, &goal.ctx);
                let arg_v = eval(g, &goal.ctx.env, &arg_t);
                proof_t = proof_t.app(arg_t);
                ty = open(&cod, arg_v, g);
                new_metas.push((m, (*dom).clone()));
            }
            _ => break,
        }
    }
    unify(g, goal.ctx.lvl(), &goal.ctx.types, &ty, &goal.ty)?;
    close_goal(g, &goal, proof_t);
    let mut goals: Vec<Goal> = new_metas
        .into_iter()
        .map(|(m, ty)| Goal {
            ctx: goal.ctx.clone(),
            ty,
            meta: m,
        })
        .collect();
    goals.append(&mut st.goals);
    st.goals = goals;
    Ok(())
}

fn refl(g: &mut Globals, st: &mut TacticState) -> KResult<()> {
    let goal = st.goals.remove(0);
    let ty = force(g, goal.ty.clone());
    let (a_ty, lhs, rhs) = match &ty {
        Value::VConst(n, sp) if n == "Eq" && sp.len() == 3 => {
            (sp[0].clone(), sp[1].clone(), sp[2].clone())
        }
        _ => {
            return Err(KError::Tactic(format!(
                "refl: goal is not an equality: {}",
                quote(g, goal.ctx.lvl(), &ty)
            )));
        }
    };
    if !conv(g, goal.ctx.lvl(), &lhs, &rhs) {
        return Err(KError::Tactic(
            "refl: sides not definitionally equal".into(),
        ));
    }
    let _ = g
        .lookup("refl")
        .ok_or_else(|| KError::Tactic("axiom 'refl' not declared".into()))?;
    let a_t = quote(g, goal.ctx.lvl(), &a_ty);
    let lhs_t = quote(g, goal.ctx.lvl(), &lhs);
    let proof = Term::Const("refl".into()).app(a_t).app(lhs_t);
    close_goal(g, &goal, proof);
    Ok(())
}

fn assumption(g: &mut Globals, st: &mut TacticState) -> KResult<()> {
    let goal = st.goals.remove(0);
    for (i, ty) in goal.ctx.types.iter().enumerate().rev() {
        if conv(g, goal.ctx.lvl(), ty, &goal.ty) {
            let idx = goal.ctx.lvl() - 1 - i;
            close_goal(g, &goal, Term::Var(idx));
            return Ok(());
        }
    }
    st.goals.insert(0, goal);
    Err(KError::Tactic("assumption: no matching hypothesis".into()))
}

fn sorry(g: &mut Globals, st: &mut TacticState) {
    let goal = st.goals.remove(0);
    let m = g.fresh_meta(goal.ty.clone());
    let proof = meta_app_in(m, &goal.ctx);
    close_goal(g, &goal, proof);
}
