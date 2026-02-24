mod common;
use common::assert_approx_eq;
use rustplex::prelude::*;

#[test]
fn test_infeasible_problem() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x);
    // Impossible: x >= 5 AND x <= 3
    model.add_constraint(x).ge(5.0);
    model.add_constraint(x).le(3.0);

    let solution = model.solve().unwrap();
    assert!(matches!(solution.status(), SolverStatus::Infeasible));
}

#[test]
fn test_unbounded_problem() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x);
    model.add_constraint(x).ge(5.0);

    let solution = model.solve().unwrap();
    assert!(matches!(solution.status(), SolverStatus::Unbounded));
}

#[test]
fn test_no_constraints_unbounded() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    model.set_objective(Maximize, x);

    let solution = model.solve().unwrap();
    assert!(matches!(solution.status(), SolverStatus::Unbounded));
}

#[test]
fn test_zero_objective() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    // Maximize 0 (Find any feasible point)
    model.set_objective(Maximize, 0.0 * x);
    model.add_constraint(x).le(5.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 0.0);
}

#[test]
fn test_redundant_constraints() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x);
    model.add_constraint(x).le(10.0); // Binding
    model.add_constraint(x).le(100.0); // Redundant

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

#[test]
fn test_numerical_stability() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    // 1M * x + 0.000001 * y
    model.set_objective(Maximize, 1_000_000.0 * x + 0.000_001 * y);
    model.add_constraint(x).le(1.0);
    model.add_constraint(y).le(1_000_000.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 1_000_001.0);
}

#[test]
fn test_complex_degeneracy_case() {
    // This case requires robust Phase 1 handling
    let mut model = Model::new();
    let x0 = model.add_variable().non_negative().continuous();
    let x1 = model.add_variable().non_negative().continuous();
    let x2 = model.add_variable().non_negative().continuous();
    let x3 = model.add_variable().non_negative().continuous();
    let x4 = model.add_variable().non_negative().continuous();
    let x5 = model.add_variable().non_negative().continuous();

    model.set_objective(Minimize, x0 + x1 + x2 + x3 + x4 + x5);

    model.add_constraint(x4 + x5).eq(3.0);
    model.add_constraint(x1 + x5).eq(5.0);
    model.add_constraint(x2 + x3 + x4).eq(4.0);
    model.add_constraint(x0 + x1 + x3).eq(7.0);

    let solution = model.solve().unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

#[test]
fn test_no_constraints_optimal() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Minimize, x);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 0.0);
}
