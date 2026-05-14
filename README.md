# tiny-kernel

A pedagogical dependent type theory kernel with elaborator and tactics, modeled on Lean. About 1800 lines of Rust, readable in an evening.

The book [Zero to QED](https://github.com/sdiehl/zero-to-qed) teaches you to use Lean. This shows how it works underneath.

## What's inside

- Core calculus: dependent functions, universes with `Prop`/`Type`/`Sort u`, `Eq`, axioms
- Normalization by evaluation with closures, de Bruijn indices for terms and levels for values
- Bidirectional elaborator (`check` / `infer`) with insertion of typed holes
- Higher-order pattern unification (Miller patterns) with partial renaming for metavariables
- A small tactic framework: `intro`, `exact`, `apply`, `refl`, `assumption`, `sorry`
- Top-level commands: `def`, `axiom`, `theorem ... by ...`, `#check`, `#eval`, `#print`

## Building

```bash
cargo build
cargo test
```

## Examples

```bash
cargo run --example demo
cargo run --example tactic
```

## Reading order

1. `src/term.rs`, `src/value.rs` -- core syntax and semantic values
2. `src/eval.rs` -- NbE: `eval`, `quote`, `force`, `whnf`, `conv`
3. `src/unify.rs` -- pattern unification with `invert` / `rename`
4. `src/elab.rs` -- bidirectional `check` / `infer`
5. `src/tactic.rs` -- goal state and tactics
6. `src/cmd.rs` -- top-level commands and prelude

## License

MIT
