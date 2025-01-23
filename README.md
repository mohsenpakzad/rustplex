# Linear Programming Solver in Rust

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

## Usage

### Input Requirements

The solver expects an LP problem in standard form, including:

1. An objective function to maximize or minimize.
2. A set of constraints.
3. Decision variable bounds.

### Example

Here is an example of how to set up and solve an LP problem:

```rust
use rustplex::core::{constraint::ConstraintSense, model::Model, objective::ObjectiveSense};

fn main() {
    let mut model = Model::new();

    let x1 = model.add_variable().name("x1").lower_bound(0.0);
    let x2 = model.add_variable().name("x2").lower_bound(0.0);
    let x3 = model.add_variable().name("x3").lower_bound(0.0);

    model.set_objective(
        ObjectiveSense::Maximize,
        x1.clone() + x2.clone() + x3.clone(),
    );

    model
        .add_constraint(x1.clone(), ConstraintSense::LessEqual, 10)
        .name("constr1");

    model
        .add_constraint(x2.clone() + x3.clone(), ConstraintSense::LessEqual, 5)
        .name("constr2");

    model.solve();

    println!("{}", model.get_solution());
}
```
