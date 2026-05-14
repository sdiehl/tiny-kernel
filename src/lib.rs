//! A pedagogical dependent type theory kernel with elaborator and tactics, modeled on Lean.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::module_name_repetitions
)]

pub mod cmd;
pub mod elab;
pub mod env;
pub mod errors;
pub mod eval;
pub mod lexer;
pub mod parse;
pub mod surface;
pub mod tactic;
pub mod term;
pub mod unify;
pub mod value;

pub use cmd::{run_cmd, run_program};
pub use errors::{KError, KResult};
pub use term::{Level, Term};
