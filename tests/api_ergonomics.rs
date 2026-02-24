mod common;
use common::assert_approx_eq;
use rustplex::prelude::*;

#[test]
fn test_integer_not_supported_error() {
    let mut model = Model::new();
    // Integers not supported in Simplex
    let x = model.add_variable().integer();
    model.set_objective(Maximize, x);

    let result = model.solve();
    assert!(matches!(result.unwrap_err(), SolverError::NonLinearNotSupported));
}

#[test]
fn test_reject_binary_variables() {
    let mut model = Model::new();
    let b = model.add_variable().binary();
    model.set_objective(Maximize, b);
    
    let result = model.solve();
    assert!(matches!(result.unwrap_err(), SolverError::NonLinearNotSupported));
}

#[test]
fn test_incremental_solving() {
    // 1. Solve initial problem
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    model.set_objective(Maximize, x);
    model.add_constraint(x).le(10.0);

    let solution1 = model.solve().unwrap();
    assert_approx_eq(solution1.objective_value().unwrap(), 10.0);

    // 2. Add tighter constraint and solve again (re-using model)
    model.add_constraint(x).le(5.0);
    
    let solution2 = model.solve().unwrap();
    assert_approx_eq(solution2.objective_value().unwrap(), 5.0);
}