# Linear Programming Solver in Rust

[![Crates.io Version](https://img.shields.io/crates/v/rustplex.svg)](https://crates.io/crates/rustplex)
[![Crates.io Page](https://img.shields.io/badge/crates.io-rustplex-orange)](https://crates.io/crates/rustplex)
[![Downloads](https://img.shields.io/crates/d/rustplex.svg)](https://crates.io/crates/rustplex)

Welcome to the **Rustplex**, a robust and efficient simplex solver implemented in Rust. This project provides tools to solve linear programming (LP) problems using the simplex method while supporting standardization, slack dictionaries, and auxiliary phases to handle infeasible and unbounded cases.

---

## Features

- **Standardization**:

  - Converts any LP problem into standard form for compatibility with the simplex algorithm.
  - Handles constraints and variable bounds automatically.

- **Simplex Solver**:

  - Implements the two-phase simplex method for solving LP problems efficiently.
  - Detects and handles infeasible or unbounded problems.

- **Configurable Solver**:

  - Supports custom tolerances and iteration limits via `SolverConfig`.
  - Offers robust numerical stability by accounting for floating-point precision errors.

- **Extensible Slack Dictionary**:

  - Efficiently manages basic and non-basic variables.
  - Allows pivot operations and tracks the objective function dynamically.

- **Detailed Solutions**:
  - Provides optimal values for decision variables.
  - Reports solver status (optimal, infeasible, unbounded, or iteration limit reached).

---

## Roadmap / TODO

Planned features and improvements for future releases:

- [ ] **Comprehensive Documentation**: Add detailed API references, architectural explanations, and practical examples to improve usability and understanding.

- [ ] **Multi-thread Architecture**: Implement parallel processing for faster solving of large-scale problems.

- [ ] **Integer & Mixed-Integer Programming (MIP)**: Add branch-and-bound support for integer and mixed-integer variables.

## Usage

### Input Requirements

The solver expects an LP problem in standard form, including:

1. An objective function to maximize or minimize.
2. A set of constraints.
3. Decision variable bounds.

### Example

Here is an example of how to set up and solve an LP problem:

```rust
use rustplex::{ConstraintSense, Model, ObjectiveSense};

fn main() {
    let mut model = Model::new();

    let x1 = model.add_variable().with_name("x1").with_lower_bound(0.0);
    let x2 = model.add_variable().with_name("x2").with_lower_bound(0.0);
    let x3 = model.add_variable().with_name("x3").with_lower_bound(0.0);

    model.set_objective(
        ObjectiveSense::Maximize,
        &x1 + &x2 + &x3,
    );

    model
        .add_constraint(&x1, ConstraintSense::LessEqual, 10)
        .with_name("constr1");

    model
        .add_constraint(&x2 + &x3, ConstraintSense::LessEqual, 5)
        .with_name("constr2");

    if model.solve().is_ok() {
        println!("{}", model.solution());
    }
}
```
