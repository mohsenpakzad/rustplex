mod common;
use common::assert_approx_eq;
use rustplex::prelude::*;

/// Klee-Minty Cube (Dimension 3)
/// Forces Simplex to visit every vertex (2^N).
/// Max 100x1 + 10x2 + x3
/// s.t. x1 <= 1, 20x1 + x2 <= 100, etc.
#[test]
fn test_klee_minty_3d() {
    let mut model = Model::new();
    let x1 = model.add_variable().non_negative().continuous();
    let x2 = model.add_variable().non_negative().continuous();
    let x3 = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, 100.0 * x1 + 10.0 * x2 + 1.0 * x3);

    model.add_constraint(x1).le(1.0);
    model.add_constraint(20.0 * x1 + x2).le(100.0);
    model.add_constraint(200.0 * x1 + 20.0 * x2 + x3).le(10000.0);

    let solution = model.solve().unwrap();
    assert_approx_eq(solution.objective_value().unwrap(), 10000.0);
}

/// Beale's Cycling Problem
/// Without Bland's Rule or perturbation, simplex may cycle here.
/// Minimize: -0.75x1 + 150x2 - 0.02x3 + 6x4
#[test]
fn test_beale_cycling() {
    let mut model = Model::new();
    let x1 = model.add_variable().non_negative().continuous();
    let x2 = model.add_variable().non_negative().continuous();
    let x3 = model.add_variable().non_negative().continuous();
    let x4 = model.add_variable().non_negative().continuous();

    // Note: Rustplex uses Maximize internally, so we might need to negate if we only support Maximize?
    // The library supports Minimize, so this is fine.
    model.set_objective(Minimize, -0.75 * x1 + 150.0 * x2 - 0.02 * x3 + 6.0 * x4);

    model.add_constraint(0.25 * x1 - 60.0 * x2 - 0.04 * x3 + 9.0 * x4).le(0.0);
    model.add_constraint(0.50 * x1 - 90.0 * x2 - 0.02 * x3 + 3.0 * x4).le(0.0);
    model.add_constraint(x3).le(1.0);

    // If this terminates, we successfully avoided infinite cycling.
    let result = model.solve();
    assert!(result.is_ok()); 
}

/// Perturbed Problem (Epsilon Test)
/// 1e-9 * x + y <= 1
/// If tolerance is too loose, this might be seen as 0*x + y <= 1
#[test]
fn test_epsilon_perturbation() {
    let mut model = Model::new();
    let x = model.add_variable().non_negative().continuous();
    let y = model.add_variable().non_negative().continuous();

    model.set_objective(Maximize, x + y);
    
    // Coefficient is slightly larger than standard epsilon (1e-10)
    let small_coeff = 1e-9; 
    model.add_constraint(small_coeff * x + y).le(1.0);

    // Optimal solution: y=0, x = 1 / 1e-9 = 1,000,000,000
    // If solver treats 1e-9 as zero, x becomes unbounded.
    let solution = model.solve().unwrap();
    
    assert!(matches!(solution.status(), SolverStatus::Optimal));
    // Check if x is large (approx 10^9)
    assert!(solution[x] > 1_000_000.0);
}