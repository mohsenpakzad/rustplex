# Rustplex

[![Crates.io Version](https://img.shields.io/crates/v/rustplex.svg)](https://crates.io/crates/rustplex)
[![Crates.io Page](https://img.shields.io/badge/crates.io-rustplex-orange)](https://crates.io/crates/rustplex)
[![Downloads](https://img.shields.io/crates/d/rustplex.svg)](https://crates.io/crates/rustplex)

A fast and efficient **Linear Programming (LP) Solver** implemented in Rust, designed to solve optimization problems using the **Simplex Algorithm**.

## ✨ Features

- **Fast & Efficient**: Optimized implementation of the **Simplex Algorithm** for solving LP problems.
- **User-Friendly API**: Designed for ease of use with a clean and intuitive API.
- **Custom Constraints & Objectives**: Define your own constraints and objective functions effortlessly.
- **Scalable & Reliable**: Suitable for large-scale linear programming problems.

## 🚀 Installation

install via Cargo:

```sh
cargo add rustplex
```

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

Output:

```
Solver Status: Optimal
Objective Value: 10.00
Variable Values: [
        Var(x2): 2.00
        Var(x3): 3.00
        Var(x1): 5.00
]
Iterations: 3
Solve Time: 18.10µs
```

## 🛠 Contributing

Contributions are welcome! Feel free to fork, submit issues, or open pull requests.

## 📄 License

This project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

---

_Developed with ❤️ in Rust._
