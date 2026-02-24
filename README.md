# Rustplex

[![Crates.io Version](https://img.shields.io/crates/v/rustplex.svg)](https://crates.io/crates/rustplex)
[![Docs.rs](https://img.shields.io/docsrs/rustplex)](https://docs.rs/rustplex)
[![License](https://img.shields.io/crates/l/rustplex)](https://github.com/mohsenpakzad/rustplex/blob/main/LICENSE-MIT)
[![Downloads](https://img.shields.io/crates/d/rustplex.svg)](https://crates.io/crates/rustplex)

**Rustplex** is a fast, type-safe, and ergonomic **Linear Programming (LP) Solver** written in pure Rust.

It uses the **Two-Phase Simplex Algorithm** to solve optimization problems, featuring a clean builder API that allows you to write mathematical constraints using standard Rust operators.

> ⚠️ **Status: Early Development**
>
> This library is currently in version `0.x`. The API is subject to **breaking changes** as we refine the solver core and introduce new features. We recommend pinning the version in your `Cargo.toml`.

---

## ✨ Key Features

* **Ergonomic Modeling**: Use standard operators (`+`, `-`, `*`) to build constraints naturally (e.g., `3*x + y <= 10`).
* **Type Safety**: Strongly typed keys (`VariableKey`, `ConstraintKey`) prevent common mistakes, like mixing up variables from different models.
* **Performance Optimized**: Internally uses **sparse sorted vectors** and an efficient `slotmap` arena for memory management, ensuring fast iteration and low allocation overhead.
* **Robust Solver**: Implements the **Two-Phase Simplex** method to automatically detect and handle infeasible or unbounded problems.
* **Standard Form Conversion**: Automatically handles complex variable bounds (e.g., free variables, ranges like `[-5, 10]`) by compiling them into standard form (`x >= 0`) behind the scenes.

---

## 🚀 Installation

Add `rustplex` to your project via Cargo:

```sh
cargo add rustplex

```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
rustplex = "0.3.0"

```

---

## 💻 Usage

Rustplex separates the **Model** (your problem definition) from the **Solver** (the math). Here is a complete example:

```rust
use rustplex::prelude::*;

fn main() -> Result<(), SolverError> {
    // 1. Initialize the model
    let mut model = Model::new();

    // 2. Define decision variables with custom bounds
    let x1 = model.add_variable().name("x1").bounds(2.0..=5.0).real();
    let x2 = model.add_variable().name("x2").non_negative().real();
    let x3 = model.add_variable().name("x3").upper_bound(1.0).real();
    let x4 = model.add_variable().name("x4").real(); // Unbounded (free)

    // 3. Set the objective function: Maximize x1 + x2 + x3 - x4
    model.set_objective(
        Maximize,
        x1 + x2 + x3 - x4,
    );

    // 4. Add constraints using natural syntax
    //    x1 + x3 <= x2
    model.add_constraint(x1 + x3).le(x2);
    
    //    x2 + x3 == 5.0
    model.add_constraint(x2 + x3).eq(5.0);

    //    x4 + x1 >= 10.0
    model.add_constraint(x4 + x1).ge(10.0);

    // 5. Solve the model
    let solution = model.solve()?;

    // 6. Inspect the results
    if solution.status().is_optimal() {
        println!("Objective Value: {}", solution.objective_value().unwrap());
        
        // Retrieve variable values safely
        println!("x1 = {}", solution[x1]);
        println!("x2 = {}", solution[x2]);
        
        // Print full detailed report
        println!("{}", model.format(&solution));
    } else {
        println!("Solver failed: {}", solution.status());
    }
    
    Ok(())
}

```

Output:

```
Objective Value: 5
x1 = 5
x2 = 5
Solver Status: Optimal
Objective Value: 5.00
Variable Values: [
        Variable(x1:cont ∈ [2, 5]): 5.00
        Variable(x2:cont ∈ [0, inf]): 5.00
        Variable(x3:cont ∈ [-inf, 1]): 0.00
        Variable(x4:cont ∈ [-inf, inf]): 5.00
]
Iterations: 8
Solve Time: 30.90µs
```

### ⚙️ Configuration

You can tune the solver's behavior (tolerances, iteration limits) via `SolverConfig`:

```rust
use rustplex::prelude::*;

let mut model = Model::new()
    .with_config(SolverConfig {
        max_iterations: 10_000,
        tolerance: 1e-8,
        ..Default::default()
    });

```

---

## 🗺️ Roadmap

We are actively working on the following features:
* [ ] **Comprehensive Documentation**: Add detailed API references, architectural explanations, and practical examples to improve usability and understanding.
* [ ] **Integer Programming (IP)**: Branch-and-bound support for integer variables.
* [ ] **Mixed-Integer Programming (MIP)**: Hybrid models with both continuous and integer variables.
* [ ] **Parallel Solving**: Multi-threaded pivoting for massive problems.

---

## 🤝 Contributing

Contributions are welcome! Whether it's reporting a bug, improving documentation, or adding a new feature, feel free to open an issue or pull request.

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'Add some amazing feature'`).
4. Push to the branch (`git push origin feature/amazing-feature`).
5. Open a Pull Request.

---

## 📄 License

This project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

---

_Developed with ❤️ in Rust._
