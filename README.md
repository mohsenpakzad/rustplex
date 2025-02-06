# Rustplex

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

## 🛠 Usage

```rust
use rustplex::core::{constraint::ConstraintSense, model::Model, objective::ObjectiveSense};

fn main() {
    let mut model = Model::new();

    let x1 = model.add_variable().name("x1").bounds(1.0..=5.0);
    let x2 = model.add_variable().name("x2").upper_bound(2.0);
    let x3 = model.add_variable().name("x3");

    model.set_objective(ObjectiveSense::Maximize, &x1 + &x2 + &x3);

    model
        .add_constraint(&x1, ConstraintSense::LessEqual, 10)
        .name("constr1");

    model
        .add_constraint(&x2 + &x3, ConstraintSense::LessEqual, 5)
        .name("constr2");

    model.solve();

    println!("{}", model.get_solution());
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
