//! The Rustplex Prelude
//!
//! Import this module to bring the most common types into scope:
//! ```
//! use rustplex::prelude::*;
//! ```

pub use crate::common::expression::LinearExpr;

pub use crate::modeling::constraint::ConstraintKey;
pub use crate::modeling::model::Model;
pub use crate::modeling::objective::ObjectiveSense::{self, Maximize, Minimize};
pub use crate::modeling::variable::VariableKey;

pub use crate::solver::config::SolverConfig;
pub use crate::solver::status::SolverStatus;

pub use crate::error::SolverError;
