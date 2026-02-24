// --- Internal Modules ---
mod common;
mod solver;
mod standard_form;

// --- Public Modules ---
pub mod modeling;
pub mod error;
pub mod prelude;

// --- API Re-exports ---
pub use crate::common::expression::LinearExpr;

pub use crate::modeling::model::Model;
pub use crate::modeling::variable::{Variable, VariableKey, VariableType};
pub use crate::modeling::constraint::{Constraint, ConstraintKey, ConstraintSense};
pub use crate::modeling::objective::{Objective, ObjectiveSense};

pub use crate::solver::config::SolverConfig;
pub use crate::solver::solution::SolverSolution;
pub use crate::solver::status::SolverStatus;
