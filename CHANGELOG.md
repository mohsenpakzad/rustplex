# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-02-24

### ⚠ BREAKING CHANGES
* **Arena Architecture**: The codebase has been migrated to an arena-based architecture using `slotmap`. Variables and Constraints are now referenced via `VariableKey` and `ConstraintKey` rather than direct references or RC pointers.
* **Stateless Solving**: `Model::solve()` no longer mutates the model to store the solution. It now returns a `Result<SolverSolution, SolverError>` directly.
* **Copy Semantics**: `ExprVariable` now requires the `Copy` trait. Reference-based arithmetic (e.g., `&var + &var`) has been simplified to value-based arithmetic.
* **Module Renames**: 
    * `core` module renamed to `modeling`.
    * `standard` module renamed to `standard_form`.
    * `SolverConfiguration` renamed to `SolverConfig`.

### 🚀 Features
* **Variable Shortcuts**: Added `.non_negative()` and `.non_positive()` helper methods to the Variable Builder.
* **Ergonomic Access**: Implemented `Index<VariableKey>` for `SolverSolution`, allowing values to be accessed via `solution[x]`.
* **Display**: Added `fmt::Display` implementations for `Model`, `Constraint`, `LinearExpr`, and `SolverSolution` for easier debugging and logging.
* **Scalar Constraints**: Implemented `From<f64>` for `LinearExpr`, allowing constraints to be built with raw numbers (e.g., `.le(14.0)`).

### ⚡ Performance
* **Sparse Vectors**: `LinearExpr` now uses sorted sparse vectors (`Vec<(Var, f64)>`) instead of `HashMap`. This improves cache locality and enables O(log N) lookups.
* **Zero Pruning**: Operations on expressions now automatically prune coefficients smaller than `1e-10`, keeping the constraint matrix sparse.
* **Operator Optimization**: Refactored generic operator implementations (Add, Sub, Mul) out of macros to improve code reuse and efficiency.

### ♻️ Refactoring
* **Standardizer**: Decoupled variable mapping logic from `Model` into a dedicated `Standardizer` struct.
* **Solver Internals**: Moved simplex-specific types out of the shared solver module and encapsulated internal logic.
* **Tests**: Replaced monolithic test files with a categorized test suite.

### 📚 Documentation
* Added a comprehensive `README.md` with quick start examples and roadmap.
* Added crate-level documentation and examples for `docs.rs`.

---

## [0.2.0] - 2026-01-4

This release introduces some architectural improvements, breaks API compatibility for better ergonomics, and resolves critical stability issues.

### ⚠ BREAKING CHANGES
* **Refactored Module Exports**: Internal modules are now re-exported for cleaner imports. You may need to update your `use` statements.
* **API Renaming**: Methods have been renamed or moved between `Model` and `StandardModel` to strictly separate concerns.

### 🐛 Fixes
* **Solver Stability**: Resolved a critical logic error that caused incorrect solutions and solver instability (Fixes #1).
* **Minimization Bug**: Corrected the objective value sign for Minimization problems (previously returned negative values).

### 🧪 Improvements
* **Testing**: Added a comprehensive test suite including unit, integration, and regression tests.
