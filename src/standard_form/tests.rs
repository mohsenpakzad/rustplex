#[cfg(test)]
mod tests {
    use crate::solver::status::SolverStatus;
    use crate::standard_form::{
        constraint::StandardConstraint, model::StandardModel, variable::StandardVariable,
    };

    // Helper for approx equality
    fn assert_approx_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "Expected {}, got {}", b, a);
    }

    /// Test 1: Basic Standard Form Problem (Optimal)
    /// Max 3x1 + 2x2
    /// 2x1 + x2 <= 10
    /// x1 + 3x2 <= 15
    #[test]
    fn test_standard_model_optimal() {
        let mut std_model = StandardModel::new();

        // 1. Add Variables (Directly, no builder)
        // x1
        let x1 = std_model.add_variable(StandardVariable::new());
        // x2
        let x2 = std_model.add_variable(StandardVariable::new());

        // 2. Set Objective
        std_model.set_objective(3.0 * x1 + 2.0 * x2);

        // 3. Add Constraints
        // 2x1 + x2 <= 10
        std_model.add_constraint(StandardConstraint::new(2.0 * x1 + x2, 10.0));
        // x1 + 3x2 <= 15
        std_model.add_constraint(StandardConstraint::new(x1 + 3.0 * x2, 15.0));

        // 4. Solve
        let result = std_model.solve();
        assert!(result.is_ok());

        let solution = result.unwrap();
        assert!(matches!(solution.status(), SolverStatus::Optimal));
        assert_approx_eq(solution.objective_value().unwrap(), 17.0);
    }

    /// Test 2: Infeasible
    /// Max x
    /// x <= -5 (Impossible since x >= 0)
    #[test]
    fn test_standard_model_infeasible() {
        let mut std_model = StandardModel::new();
        let x = std_model.add_variable(StandardVariable::new());

        std_model.set_objective(1.0 * x);

        // Constraint: x <= -5
        std_model.add_constraint(StandardConstraint::new(1.0 * x, -5.0));

        let result = std_model.solve();
        let solution = result.unwrap();
        assert!(matches!(solution.status(), SolverStatus::Infeasible));
    }
}
