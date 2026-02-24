mod common;
use common::assert_approx_eq;
use rustplex::prelude::*;
use std::time::Instant;

#[test]
fn test_scale_hypercube_50_vars() {
    let mut model = Model::new();
    let n = 50;
    let mut objective = LinearExpr::new();

    // Maximize sum(x_i) s.t. x_i <= 1
    for _ in 0..n {
        let v = model.add_variable().non_negative().continuous();
        model.add_constraint(v).le(1.0);
        objective.add_term(v, 1.0);
    }
    model.set_objective(Maximize, objective);

    let start = Instant::now();
    let solution = model.solve().unwrap();
    let duration = start.elapsed();

    println!("Solved 50-dim hypercube in {:?}", duration);

    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 50.0);
}

#[test]
fn test_scale_hypercube_100_vars() {
    let mut model = Model::new();
    let n = 100;
    let mut objective = LinearExpr::new();

    for _ in 0..n {
        let v = model.add_variable().non_negative().continuous();
        model.add_constraint(v).le(1.0);
        objective.add_term(v, 1.0);
    }
    model.set_objective(Maximize, objective);

    let start = Instant::now();
    let solution = model.solve().unwrap();
    let duration = start.elapsed();

    println!("Solved 100-dim hypercube in {:?}", duration);

    assert!(matches!(solution.status(), SolverStatus::Optimal));
    assert_approx_eq(solution.objective_value().unwrap(), 100.0);
}
