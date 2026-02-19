use std::{fmt, time};
use slotmap::{SecondaryMap, Key};

use super::status::SolverStatus;

/// The result of a solved optimization model.
#[derive(Debug, Clone)]
pub struct SolverSolution<V: Key> {
    status: SolverStatus,
    objective_value: Option<f64>,
    variable_values: Option<SecondaryMap<V, f64>>,
    iterations: u32,
    solve_time: time::Duration,
}

impl<V: Key> SolverSolution<V> {
    pub fn new(
        status: SolverStatus,
        objective_value: f64,
        variable_values: SecondaryMap<V, f64>,
        iterations: u32,
        solve_time: time::Duration,
    ) -> Self {
        Self {
            status,
            objective_value: Some(objective_value),
            variable_values: Some(variable_values),
            iterations,
            solve_time,
        }
    }

    pub fn new_infeasible(iterations: u32, solve_time: time::Duration) -> Self {
        Self {
            status: SolverStatus::Infeasible,
            objective_value: None,
            variable_values: None,
            iterations,
            solve_time,
        }
    }

    /// Returns the final status of the solver (e.g., Optimal, Infeasible).
    pub fn status(&self) -> &SolverStatus {
        &self.status
    }

    /// Returns the final objective value.
    ///
    /// If the status is not Optimal, this value might be meaningless (e.g., infinity).
    pub fn objective_value(&self) -> &Option<f64> {
        &self.objective_value
    }

    pub fn variable_values(&self) -> &Option<SecondaryMap<V, f64>> {
        &self.variable_values
    }

    /// Returns the number of simplex iterations performed.
    pub fn iterations(&self) -> &u32 {
        &self.iterations
    }

    /// Returns the time taken to solve the problem.
    pub fn solve_time(&self) -> &std::time::Duration {
        &self.solve_time
    }
}

impl<V: Key> Default for SolverSolution<V> {
    fn default() -> Self {
        Self {
            status: SolverStatus::NotSolved,
            objective_value: None,
            variable_values: None,
            iterations: 0,
            solve_time: time::Duration::ZERO,
        }
    }
}

impl<V: fmt::Display + Key> fmt::Display for SolverSolution<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Solver Status: {:?}", self.status)?;
        if self.status.is_optimal() {
            writeln!(
                f,
                "Objective Value: {:.2}",
                self.objective_value.unwrap_or(0.0)
            )?;
        } else {
            writeln!(f, "Objective Value: {:?}", self.objective_value)?;
        }

        if let Some(ref vars) = self.variable_values {
            writeln!(f, "Variable Values: [")?;
            for (var, value) in vars {
                writeln!(f, "\t{}: {:.2}", var, value)?;
            }
            writeln!(f, "]")?;
        } else {
            writeln!(f, "Variable Values: None")?;
        }
        writeln!(f, "Iterations: {}", self.iterations)?;
        write!(f, "Solve Time: {:.2?}", self.solve_time)?;
        Ok(())
    }
}
