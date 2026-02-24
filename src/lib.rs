//! # Rustplex
//!
//! `rustplex` is a linear programming (LP) solver written in Rust, designed with ergonomics and correctness in mind.
//! It provides a type-safe API for modeling optimization problems and solving them using the Simplex algorithm.
//!
//! ## Key Features
//!
//! * **Ergonomic API:** Use standard Rust operators (`+`, `-`, `*`) to build linear expressions naturally.
//! * **Type Safety:** Strongly typed keys (`VariableKey`, `ConstraintKey`) prevent mixing up variables and constraints.
//! * **Builder Pattern:** Fluent interface for defining variables and constraints.
//! * **Encapsulation:** Solvers are isolated from the model definition, allowing for future expansion (e.g., Integer Programming).
//!
//! ## Quick Start
//!
//! Add `rustplex` to your `Cargo.toml`. Then, you can define and solve a problem like this:
//!
//! ```rust
//! use rustplex::prelude::*;
//!
//! fn main() -> Result<(), SolverError> {
//!     // 1. Create a model
//!     let mut model = Model::new();
//!
//!     // 2. Define variables
//!     let x1 = model.add_variable().name("x1").non_negative().continuous();
//!     let x2 = model.add_variable().name("x2").bounds(0.0..=10.0).continuous();
//!
//!     // 3. Set objective: Maximize x1 + x2
//!     model.set_objective(Maximize, x1 + x2);
//!
//!     // 4. Add constraints
//!     // 2*x1 + x2 <= 10
//!     model.add_constraint(2.0 * x1 + x2).le(10.0);
//!
//!     // 5. Solve
//!     let solution = model.solve()?;
//!
//!     if solution.status() == &SolverStatus::Optimal {
//!         println!("Objective Value: {}", solution.objective_value().unwrap());
//!         println!("x1: {}", solution[x1]);
//!         println!("x2: {}", solution[x2]);
//!     }
//!
//!     Ok(())
//! }
//! ```

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
