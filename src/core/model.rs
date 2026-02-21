use std::fmt;
use slotmap::{DenseSlotMap, SecondaryMap};

use crate::{
    core::{
        constraint::{ConstraintKey, Constraint, ConstraintBuilder},
        expression::LinearExpr,
        objective::{Objective, ObjectiveSense},
        variable::{VariableKey, Variable, VariableType, VariableBuilder},
    },
    error::SolverError,
    simplex::{config::SolverConfiguration, solution::SolverSolution, status::SolverStatus},
    standard::model::StandardModel,
};

#[derive(Debug)]
pub struct Model {
    variables: DenseSlotMap<VariableKey, Variable>,
    constraints: DenseSlotMap<ConstraintKey, Constraint>,
    objective: Option<Objective>,
    solution: SolverSolution<VariableKey>,
    config: SolverConfiguration,
}

impl Model {
    /// Creates a new, empty model with default settings.
    pub fn new() -> Self {
        Self {
            variables: DenseSlotMap::with_key(),
            constraints: DenseSlotMap::with_key(),
            objective: None,
            solution: SolverSolution::default(),
            config: SolverConfiguration::default(),
        }
    }

    // --- Configuration Methods ---

    pub fn with_config(mut self, config: SolverConfiguration) -> Self {
        self.config = config;
        self
    }

    /// Sets the maximum number of iterations for the solver.
    ///
    /// Default is 10,000.
    pub fn set_max_iterations(&mut self, max_iterations: u32) {
        self.config.max_iterations = max_iterations;
    }

    /// Sets the numerical tolerance (epsilon) for the solver.
    ///
    /// Default is 1e-10. Values smaller than this are treated as zero.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.config.tolerance = tolerance;
    }

    // --- Builder Methods ---

    pub fn add_variable(&mut self) -> VariableBuilder<'_> {
        VariableBuilder::new(&mut self.variables)
    }

    pub fn add_constraint(&mut self, lhs: impl Into<LinearExpr<VariableKey>>) -> ConstraintBuilder<'_> {
        ConstraintBuilder::new(&mut self.constraints, lhs.into())
    }

    pub fn set_objective(&mut self, sense: ObjectiveSense, expression: impl Into<LinearExpr<VariableKey>>) {
        self.objective = Some(Objective::new(sense, expression.into()));
    }

    fn is_lp(&self) -> bool {
        !self
            .variables
            .values()
            .any(|variable| !matches!(variable.var_type(), VariableType::Continuous))
    }

    pub fn to_standard(&self) -> StandardModel {
        StandardModel::from_model(&self)
    }

    pub fn solve(&mut self) -> Result<(), SolverError> {
        if !self.is_lp() {
            return Err(SolverError::NonLinearNotSupported);
        } else if self.variables.is_empty() {
            return Err(SolverError::NoVariables);
        } else if self.objective.is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        let mut standardized_model =
            StandardModel::from_model(&self).with_config(self.config);

        standardized_model.solve()?;

        self.solution = self.construct_solution_from_standard_model(&standardized_model);

        Ok(())
    }

    /// Internal helper to translate the standardized solution back to the user model context.
    /// Handles variable mapping and objective sign correction.
    fn construct_solution_from_standard_model(
        &self,
        std_model: &StandardModel,
    ) -> SolverSolution<VariableKey> {
        let std_solution = std_model.solution();

        if matches!(std_solution.status(), SolverStatus::Infeasible) {
            return SolverSolution::new_infeasible(
                *std_solution.iterations(),
                *std_solution.solve_time(),
            );
        }

        // 1. Map values back to original variables
        // We iterate over the original variables in the model and query the standard model for their values.
        let variable_values: SecondaryMap<VariableKey, f64> = self
            .variables
            .keys()
            .map(|var_key| (var_key, std_model.get_variable_value(var_key).unwrap())) // Added unwrap_or(0.0)
            .collect();

        // 2. Handle Objective Value and Sign
        // If the objective was Minimize, we must negate the result (since Simplex solved for Max -Z)
        let mut objective_value = std_solution.objective_value().unwrap();

        if let Some(obj) = &self.objective {
            if matches!(obj.sense(), ObjectiveSense::Minimize) {
                objective_value = -objective_value;
            }
        }

        // 3. Construct the new solution object
        SolverSolution::new(
            std_solution.status().clone(),
            objective_value,
            variable_values,
            *std_solution.iterations(),
            *std_solution.solve_time(),
        )
    }

    pub fn variables(&self) -> &DenseSlotMap<VariableKey, Variable> {
        &self.variables
    }

    pub fn constraints(&self) -> &DenseSlotMap<ConstraintKey, Constraint> {
        &self.constraints
    }

    pub fn objective(&self) -> &Option<Objective> {
        &self.objective
    }

    pub fn solution(&self) -> &SolverSolution<VariableKey> {
        &self.solution
    }

    pub fn config(&self) -> &SolverConfiguration {
        &self.config
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the objective, if it exists
        match &self.objective {
            Some(objective) => {
                writeln!(f, "Objective: {}", objective)?;
            }
            None => {
                writeln!(f, "Objective: None")?;
            }
        }

        // Display the constraints
        writeln!(f, "Constraints: [")?;
        for constraint in self.constraints.values() {
            writeln!(f, "\t{},", constraint)?;
        }
        writeln!(f, "]")?;

        // Display the variables
        writeln!(f, "Variables: [")?;
        for variable in self.variables.values() {
            writeln!(f, "\t{},", variable)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}
