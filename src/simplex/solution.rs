use std::{collections::HashMap, time};

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
