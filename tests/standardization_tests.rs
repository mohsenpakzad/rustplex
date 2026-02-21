mod common;

use common::assert_approx_eq;
use rustplex::{standard::model::StandardModel, SolverStatus};

/// Test 1: Basic Standard Form Problem
///
/// Problem Definition (Standard Form):
/// var x1 >= 0;
/// var x2 >= 0;
///
/// maximize z: 3x1 + 2x2;
///
/// subject to c1: 2x1 + x2 <= 10;
/// subject to c2: x1 + 3x2 <= 15;
///
/// Solution Description:
/// We are solving directly on the StandardModel.
/// Intersection of c1 and c2:
/// From c1: x2 = 10 - 2x1
/// Sub into c2: x1 + 3(10 - 2x1) = 15
/// x1 + 30 - 6x1 = 15
/// -5x1 = -15 => x1 = 3
/// x2 = 10 - 6 = 4
/// Objective Z = 3(3) + 2(4) = 9 + 8 = 17.
#[test]
fn test_standard_model_optimal() {
    let mut std_model = StandardModel::new();

    // 1. Define Standard Variables (implicitly >= 0)
    let x1 = std_model.build_variable().continuous();
    let x2 = std_model.build_variable().continuous();

    // 2. Define Objective (implicitly Maximize)
    std_model.set_objective(3.0 * &x1 + 2.0 * &x2);

    // 3. Define Constraints (implicitly LHS <= RHS)
    // 2x1 + x2 <= 10
    std_model.build_constraint(2.0 * x1 + x2).le(10.0);
    // x1 + 3x2 <= 15
    std_model.build_constraint(&x1 + 3.0 * &x2).le(15.0);

    // 4. Solve
    let result = std_model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 17.0);
}

/// Test 2: Infeasible Standard Form Problem
///
/// Problem Definition (Standard Form):
/// var x >= 0;
///
/// maximize z: x;
///
/// subject to c1: x <= -5;
///
/// Solution Description:
/// Standard variables are by definition x >= 0.
/// The constraint requires x <= -5.
/// No value exists that satisfies both.
#[test]
fn test_standard_model_infeasible() {
    let mut std_model = StandardModel::new();

    let x = std_model.build_variable().continuous();

    std_model.set_objective(1.0 * &x);

    // Constraint: x <= -5
    std_model.build_constraint(1.0 * &x).le(-5.0);

    let result = std_model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Infeasible));
    assert!(solution.objective_value().is_none());
}

/// Test 3: Unbounded Standard Form Problem
///
/// Problem Definition (Standard Form):
/// var x >= 0;
///
/// maximize z: x;
///
/// subject to c1: -x <= 5;
///
/// Solution Description:
/// Constraint -x <= 5 implies x >= -5.
/// Combined with x >= 0 (implicit), the valid region is x >= 0.
/// Since we want to maximize x, we can increase it infinitely.
#[test]
fn test_standard_model_unbounded() {
    let mut std_model = StandardModel::new();

    let x = std_model.build_variable().continuous();

    std_model.set_objective(1.0 * &x);

    // Constraint: -x <= 5
    // Note: In standard form, coefficients can be negative.
    std_model.build_constraint(-1.0 * &x).le(5.0);

    assert!(std_model.solve().is_ok());

    let result = std_model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Unbounded));
}

/// Test 4: Artificial Variable Necessity (Phase 1 Trigger)
///
/// Problem Definition (Standard Form):
/// var x >= 0;
///
/// maximize z: -x;
///
/// subject to c1: -x <= -10;
///
/// Solution Description:
/// -x <= -10 multiply by -1 => x >= 10.
/// This looks simple, but in the Simplex Dictionary (Standard Form),
/// RHS must generally be non-negative for a feasible starting basis.
/// Here RHS is -10. This forces the solver to enter Phase 1
/// to find a valid starting dictionary.
/// Optimal solution: Smallest x >= 10 is x=10.
/// Maximize -x => -10.
#[test]
fn test_standard_model_needs_phase_1() {
    let mut std_model = StandardModel::new();

    let x = std_model.build_variable().continuous();

    // Maximize -x
    std_model.set_objective(-1.0 * &x);

    // Constraint: -x <= -10 (equivalent to x >= 10)
    std_model.build_constraint(-1.0 * &x).le(-10.0);

    let result = std_model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), -10.0);
}
