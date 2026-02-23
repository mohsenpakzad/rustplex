mod common;

use common::assert_approx_eq;
use rustplex::{
    modeling::{
        expression::LinearExpr, 
        model::Model,
        objective::ObjectiveSense::{Maximize, Minimize}
    },
    simplex::status::SolverStatus,
    error::SolverError,
};

/// Test Case 1: Standard Maximization Problem
///
/// Problem Definition:
/// var x >= 0;
/// var y >= 0;
///
/// maximize z: 3x + 4y;
///
/// subject to c1: x + 2y <= 14;
/// subject to c2: 3x - y <= 0;
/// subject to c3: x - y <= 2;
///
/// Solution Description:
/// This is a 2D geometry problem.
/// Intersection of c1 (x + 2y = 14) and c2 (3x = y):
/// x + 2(3x) = 14 => 7x = 14 => x = 2.
/// y = 6.
/// Check c3: 2 - 6 = -4 <= 2 (Valid).
/// Objective Z = 3(2) + 4(6) = 6 + 24 = 30.
#[test]
fn test_maximization_standard() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, 3.0 * x + 4.0 * y);

    model.add_constraint(x + 2.0 * y).le(14.0);
    model.add_constraint(3.0 * x - y).le(0.0);
    model.add_constraint(x - y).le(2.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 30.0);

    let vars = solution.variable_values().as_ref().unwrap();
    assert_approx_eq(*vars.get(x).unwrap(), 2.0);
    assert_approx_eq(*vars.get(y).unwrap(), 6.0);
}

/// Test Case 2: Minimization Problem
///
/// Problem Definition:
/// var x >= 0;
/// var y >= 0;
///
/// minimize z: 2x + 3y;
///
/// subject to c1: x + y >= 10;
/// subject to c2: x <= 8;
/// subject to c3: y <= 12;
///
/// Solution Description:
/// To minimize cost (2x + 3y), we prioritize the cheaper variable x.
/// Maximize x within bounds: set x = 8 (from c2).
/// Satisfy c1: 8 + y >= 10 => y >= 2.
/// To minimize z, choose smallest valid y: y = 2.
/// Check c3: 2 <= 12 (Valid).
/// Objective Z = 2(8) + 3(2) = 16 + 6 = 22.
#[test]
fn test_minimization_standard() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Minimize, 2.0 * x + 3.0 * y);

    model.add_constraint(x + y).ge(10.0);
    model.add_constraint(x).le(8.0);
    model.add_constraint(y).le(12.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 22.0);
}

/// Test Case 3: Infeasible Problem
///
/// Problem Definition:
/// var x >= 0;
///
/// maximize z: x;
///
/// subject to c1: x >= 5;
/// subject to c2: x <= 3;
///
/// Solution Description:
/// No number exists that is both >= 5 and <= 3.
/// The problem is infeasible.
#[test]
fn test_infeasible_problem() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x);

    model.add_constraint(x).ge(5.0);
    model.add_constraint(x).le(3.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Infeasible));
    assert!(solution.objective_value().is_none());
}

/// Test Case 4: Unbounded Problem
///
/// Problem Definition:
/// var x >= 0;
///
/// maximize z: x;
///
/// subject to c1: x >= 5;
///
/// Solution Description:
/// x can increase indefinitely while satisfying x >= 5.
/// The objective value tends towards infinity.
#[test]
fn test_unbounded_problem() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x);

    model.add_constraint(x).ge(5.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Unbounded));
}

/// Test Case 5: Equality Constraints
///
/// Problem Definition:
/// var x >= 0;
/// var y >= 0;
///
/// maximize z: x + y;
///
/// subject to c1: 2x + y == 10;
///
/// Solution Description:
/// From c1: y = 10 - 2x.
/// Substitute into Z: Z = x + (10 - 2x) = 10 - x.
/// To maximize Z (10 - x) subject to x >= 0, we must minimize x.
/// Smallest x = 0.
/// Then y = 10.
/// Objective Z = 10.
#[test]
fn test_equality_constraint() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x + y);

    model.add_constraint(2.0 * x + y).eq(10.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

/// Test Case 6: Boxed Variables
///
/// Problem Definition:
/// var x >= 2;
/// var x <= 5;
///
/// maximize z: x;
///
/// Solution Description:
/// The variable is explicitly bounded between 2 and 5.
/// To maximize x, we simply pick the upper bound.
/// Objective Z = 5.
#[test]
fn test_boxed_variables() {
    let mut model = Model::new();
    let x = model.add_variable().bounds(2.0..=5.0).continuous();

    model.set_objective(Maximize, x);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 5.0);
}

/// Test Case 7: Free / Negative Variables
///
/// Problem Definition:
/// var x >= -5;
///
/// minimize z: x;
///
/// subject to c1: x <= 10;
///
/// Solution Description:
/// We want the smallest possible value for x.
/// The lower bound is explicitly -5.
/// Objective Z = -5.
#[test]
fn test_negative_variables() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(-5.0).continuous();

    model.set_objective(Minimize, x);
    model.add_constraint(x).le(10.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), -5.0);
}

/// Test Case 8: Redundant Constraints
///
/// Problem Definition:
/// var x1 >= 0;
/// var x2 >= 0;
///
/// maximize z: x1 + x2;
///
/// subject to c1: x1 + x2 <= 10;
/// subject to c2: x1 + x2 <= 12;
///
/// Solution Description:
/// c2 is looser than c1. c1 is the binding constraint.
/// We maximize sum(x) subject to sum <= 10.
/// Objective Z = 10.
#[test]
fn test_redundant_constraints() {
    let mut model = Model::new();
    let x1 = model.add_variable().lower_bound(0.0).continuous();
    let x2 = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x1 + x2);

    model.add_constraint(x1 + x2).le(10.0);
    model.add_constraint(x1 + x2).le(12.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

/// Test Case 9: Integer Guard
///
/// Problem Definition:
/// var x integer;
///
/// maximize z: x;
///
/// Solution Description:
/// This is an API validity test. The Simplex solver currently only supports
/// continuous variables. It should return a `NonLinearNotSupported` error.
#[test]
fn test_integer_not_supported() {
    let mut model = Model::new();
    let _x = model.add_variable().integer();

    model.set_objective(Maximize, _x);

    let result = model.solve();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SolverError::NonLinearNotSupported
    ));
}

/// Test Case 10: Complex Degeneracy Case
///
/// Problem Definition:
/// var x0 >= 0;
/// var x1 >= 0;
/// var x2 >= 0;
/// var x3 >= 0;
/// var x4 >= 0;
/// var x5 >= 0;
///
/// minimize z: x0 + x1 + x2 + x3 + x4 + x5;
///
/// subject to c0: x4 + x5 == 3;
/// subject to c1: x1 + x5 == 5;
/// subject to c2: x2 + x3 + x4 == 4;
/// subject to c3: x0 + x1 + x3 == 7;
///
/// Solution Description:
/// This system requires Phase 1 to solve.
/// Through algebraic substitution, the objective function simplifies to:
/// Z = 11 + x5 - x3 (constrained by system bounds).
/// Minimizing Z yields an objective value of 10.
/// A valid solution point is: x0=1, x1=5, x2=0, x3=1, x4=3, x5=0 (Sum=10).
#[test]
fn test_complex_degeneracy_case() {
    let mut model = Model::new();

    let x0 = model.add_variable().lower_bound(0f64).continuous();
    let x1 = model.add_variable().lower_bound(0f64).continuous();
    let x2 = model.add_variable().lower_bound(0f64).continuous();
    let x3 = model.add_variable().lower_bound(0f64).continuous();
    let x4 = model.add_variable().lower_bound(0f64).continuous();
    let x5 = model.add_variable().lower_bound(0f64).continuous();

    model.set_objective(Minimize, x0 + x1 + x2 + x3 + x4 + x5);

    model.add_constraint(x4 + x5).eq(3.0);
    model.add_constraint(x1 + x5).eq(5.0);
    model.add_constraint(x2 + x3 + x4).eq(4.0);
    model.add_constraint(x0 + x1 + x3).eq(7.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);
}

/// Test: Solve, Add Constraint, Solve Again
/// 1. Max x, x <= 10. (Result should be 10)
/// 2. Add constraint x <= 5.
/// 3. Solve again. (Result should be 5)
#[test]
fn test_incremental_solving() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x);
    model.add_constraint(x).le(10.0);

    // First Run
    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 10.0);

    // Modify Model
    model.add_constraint(x).le(5.0);

    // Second Run
    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 5.0);
}

/// Test: Mixing large and small coefficients
/// Max 1000000x + 0.000001y
/// x <= 1
/// y <= 1000000
#[test]
fn test_numerical_stability() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    // Big coefficient vs Small coefficient
    model.set_objective(Maximize, 1_000_000.0 * x + 0.000_001 * y);

    model.add_constraint(x).le(1.0);
    model.add_constraint(y).le(1_000_000.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    // Expected: 1,000,000 * 1 + 0.000001 * 1,000,000 = 1,000,000 + 1 = 1,000,001
    assert_approx_eq(solution.objective_value().unwrap(), 1_000_001.0);
}

/// Test: Zero Objective
/// Feasibility check: Just find ANY valid point.
#[test]
fn test_zero_objective() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    // Maximize 0 (Find any feasible solution)
    model.set_objective(Maximize, 0.0 * x);
    model.add_constraint(x).le(5.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    // Objective value should be strictly 0.0
    assert_approx_eq(solution.objective_value().unwrap(), 0.0);
}

/// Test: Variables not involved in Objective
/// Max x
/// x + y <= 10
/// (y is free to be anything, shouldn't crash the solver)
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

/// Test: N-dimensional Hypercube
/// Maximize sum(x_i) subject to x_i <= 1 for all i.
/// Result should be N.
#[test]
fn test_scale_hypercube_50_vars() {
    let mut model = Model::new();
    let n = 50;
    let mut vars = Vec::new();
    let mut objective = LinearExpr::new();

    // Create 50 variables
    for _ in 0..n {
        let v = model
            .add_variable()
            .lower_bound(0.0)
            .continuous();

        // Add constraint x_i <= 1
        model.add_constraint(v).le(1.0);

        // Add to objective: + 1.0 * x_i
        objective.add_term(v.clone(), 1.0);
        vars.push(v);
    }

    model.set_objective(Maximize, objective);

    let start = std::time::Instant::now();
    let result = model.solve();
    let duration = start.elapsed();

    println!("Solved 50 vars in {:?}", duration);

    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 50.0);
}

/// Test: Model with variables but NO constraints (Unbounded)
/// Maximize x (x >= 0) -> Infinity
#[test]
fn test_no_constraints_unbounded() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Unbounded));
}

/// Test: Model with variables but NO constraints (Optimal)
/// Minimize x (x >= 0) -> 0
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

/// Test: Klee-Minty Cube (Dimension 3)
/// A pathological case that forces the simplex algorithm to visit every vertex.
///
/// Maximize 100x1 + 10x2 + 1x3
/// Subject to:
/// 1x1 <= 1
/// 20x1 + 1x2 <= 100
/// 200x1 + 20x2 + 1x3 <= 10000
/// x >= 0
///
/// Solution: x1=0, x2=0, x3=10000. Obj = 10000.
#[test]
fn test_klee_minty_3d() {
    let mut model = Model::new();
    let x1 = model.add_variable().lower_bound(0.0).continuous();
    let x2 = model.add_variable().lower_bound(0.0).continuous();
    let x3 = model.add_variable().lower_bound(0.0).continuous();

    // Coefficients: 10^2, 10^1, 10^0
    model.set_objective(
        Maximize,
        100.0 * x1 + 10.0 * x2 + 1.0 * x3,
    );

    // Recursive constraints
    model.add_constraint(x1).le(1.0);
    model.add_constraint(20.0 * x1 + x2).le(100.0);
    model.add_constraint(200.0 * x1 + 20.0 * x2 + x3).le(10000.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 10000.0);

    // Note: If you print solution.iterations(), it will be high (7 for Dim 3).
}

/// Test: Binary Variable Rejection
/// The solver currently only supports LP (Continuous).
/// It should explicitly error if a Binary variable is detected.
#[test]
fn test_reject_binary_variables() {
    let mut model = Model::new();

    // Create a binary variable (0 or 1)
    let b = model.add_variable().binary();

    model.set_objective(Maximize, b);
    model.add_constraint(b).le(0.5);

    // This should fail because is_lp() returns false for Binary types
    let result = model.solve();

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SolverError::NonLinearNotSupported
    ));
}

/// Test: Precision with fractions
/// Max x + y
/// 3x + y <= 1
/// Result: x=0, y=1 (Obj=1) OR x=1/3, y=0 (Obj=0.33..) -> y is better.
#[test]
fn test_fractional_coefficients() {
    let mut model = Model::new();
    let x = model.add_variable().lower_bound(0.0).continuous();
    let y = model.add_variable().lower_bound(0.0).continuous();

    model.set_objective(Maximize, x + y);

    // 3x + 3y <= 1 (Wait, let's make it 3x + y <= 1)
    // If we maximize x+y:
    // Corner 1: x=0, y=1 => Obj = 1
    // Corner 2: x=1/3, y=0 => Obj = 0.33
    // Corner 3: x=0, y=0
    model.add_constraint(3.0 * x + y).le(1.0);

    let result = model.solve();
    assert!(result.is_ok());

    let solution = result.unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 1.0);
}
