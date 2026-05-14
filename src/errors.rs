use thiserror::Error;

use crate::term::{MetaId, Name, Term};

#[derive(Debug, Error)]
pub enum KError {
    #[error("parse: {0}")]
    Parse(String),

    #[error("unbound name: {0}")]
    Unbound(Name),

    #[error("type mismatch:\n  expected: {expected}\n  actual:   {actual}")]
    TypeMismatch { expected: Term, actual: Term },

    #[error("expected function, got: {0}")]
    NotFn(Term),

    #[error("expected type, got: {0}")]
    NotType(Term),

    #[error("cannot solve metavariable ?{0}")]
    UnsolvedMeta(MetaId),

    #[error("occurs check failed: ?{0}")]
    Occurs(MetaId),

    #[error("escape: variable {0} escapes its scope")]
    Escape(usize),

    #[error("non-pattern unification: {0}")]
    NonPattern(String),

    #[error("tactic failed: {0}")]
    Tactic(String),

    #[error("{0}")]
    Other(String),
}

pub type KResult<T> = Result<T, KError>;
