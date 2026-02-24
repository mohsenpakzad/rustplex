use crate::{modeling::variable::VariableKey, solver::status::SolverStatus};
use slotmap::{Key, SecondaryMap};
use std::{fmt, ops::Index, time};

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

    /// Returns the value of a specific variable.
    ///
    /// Returns `0.0` if the variable is not found in the solution (e.g., it was presolved out
    /// or is implicit).
    pub fn value(&self, var_key: V) -> f64 {
        self.variable_values
            .as_ref()
            .and_then(|map| map.get(var_key))
            .copied()
            .unwrap_or(0.0)
    }
}

/// Allows indexing notation `solution[x]` to retrieve variable values.
impl Index<VariableKey> for SolverSolution<VariableKey> {
    type Output = f64;

    fn index(&self, var_key: VariableKey) -> &Self::Output {
        match self.variable_values.as_ref() {
            Some(map) => map.get(var_key).unwrap_or(&0.0),
            None => &0.0,
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
