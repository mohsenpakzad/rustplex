use std::{collections::HashMap, fmt, time};

use super::status::SolverStatus;

#[derive(Debug, Clone)]
pub struct SolverSolution<V> {
    status: SolverStatus,
    objective_value: Option<f64>,
    variable_values: Option<HashMap<V, f64>>,
    iterations: u32,
    solve_time: time::Duration,
}

impl<V> SolverSolution<V> {
    pub fn new(
        status: SolverStatus,
        objective_value: f64,
        variable_values: HashMap<V, f64>,
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

    pub fn clone_with_new_variable_type<U>(
        &self,
        variable_values: Option<HashMap<U, f64>>,
    ) -> SolverSolution<U> {
        SolverSolution {
            status: self.status.clone(),
            objective_value: self.objective_value,
            variable_values,
            iterations: self.iterations,
            solve_time: self.solve_time,
        }
    }

    pub fn get_status(&self) -> &SolverStatus {
        &self.status
    }

    pub fn get_objective_value(&self) -> &Option<f64> {
        &self.objective_value
    }

    pub fn get_variable_values(&self) -> &Option<HashMap<V, f64>> {
        &self.variable_values
    }
}

impl<V> Default for SolverSolution<V> {
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

impl<V: fmt::Display + Eq + std::hash::Hash> fmt::Display for SolverSolution<V> {
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
