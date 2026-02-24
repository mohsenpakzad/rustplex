mod common;
use common::assert_approx_eq;
use rustplex::prelude::*;

#[test]
fn test_maximization_standard() {
    let mut model = Model::new();
    // 2x1 + x2 <= 14, etc.
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, 3.0 * x + 4.0 * y);
    model.add_constraint(x + 2.0 * y).le(14.0);
    model.add_constraint(3.0 * x - y).le(0.0);
    model.add_constraint(x - y).le(2.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 30.0);
    assert_approx_eq(solution[x], 2.0);
    assert_approx_eq(solution[y], 6.0);
}

#[test]
fn test_minimization_standard() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    model.set_objective(Minimize, 2.0 * x + 3.0 * y);

    model.add_constraint(x + y).ge(10.0);
    model.add_constraint(x).le(8.0);
    model.add_constraint(y).le(12.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 22.0);
}

#[test]
fn test_equality_constraint() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x + y);
    // 2x + y = 10 -> Force intersection line
    model.add_constraint(2.0 * x + y).eq(10.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

#[test]
fn test_boxed_variables() {
    let mut model = Model::new();
    // Explicit range [2, 5]
    let x = model.add_variable().bounds(2.0..=5.0).continuous();
    model.set_objective(Maximize, x);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 5.0);
}

#[test]
fn test_negative_variables() {
    let mut model = Model::new();
    // Lower bound is negative
    let x = model.add_variable().lower_bound(-5.0).continuous();

    model.set_objective(Minimize, x);
    model.add_constraint(x).le(10.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), -5.0);
}

#[test]
fn test_fractional_coefficients() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x + y);
    // 3x + y <= 1
    model.add_constraint(3.0 * x + y).le(1.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 1.0);
}

#[test]
fn test_unused_variable_in_objective() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x); // y is ignored here
    model.add_constraint(x + y).le(10.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}
