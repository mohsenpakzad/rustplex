use std::{collections::HashMap, fmt};

use crate::{
    core::{
        constraint::{Constr, ConstraintSense},
        expression::LinearExpr,
        objective::{Objective, ObjectiveSense},
        variable::Var,
    },
    error::SolverError,
    simplex::{config::SolverConfig, solution::SolverSolution, status::SolverStatus},
    standardization::standard_model::StandardModel,
};

use super::variable::VariableType;

#[derive(Debug, Default)]
pub struct Model {
    variables: Vec<Var>,
    constraints: Vec<Constr>,
    objective: Option<Objective>,
    solution: SolverSolution<Var>,
    config: Option<SolverConfig>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: SolverConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn add_variable(&mut self) -> Var {
        let var = Var::new();
        self.variables.push(var.clone());
        var
    }

    pub fn add_constraint(
        &mut self,
        lhs: impl Into<LinearExpr<Var>>,
        sense: ConstraintSense,
        rhs: impl Into<LinearExpr<Var>>,
    ) -> Constr {
        let constr = Constr::new(lhs.into(), sense, rhs.into());
        self.constraints.push(constr.clone());
        constr
    }

    pub fn set_objective(&mut self, sense: ObjectiveSense, expression: impl Into<LinearExpr<Var>>) {
        self.objective = Some(Objective::new(sense, expression.into()));
    }

    pub fn is_lp(&self) -> bool {
        !self
            .variables
            .iter()
            .any(|var| !matches!(var.var_type(), VariableType::Continuous))
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
            StandardModel::from_model(&self).with_config(self.config.clone().unwrap_or_default());

        standardized_model.solve()?;

        self.solution = self.construct_solution_from_standard_model(&standardized_model);

        Ok(())
    }

    /// Internal helper to translate the standardized solution back to the user model context.
    /// Handles variable mapping and objective sign correction.
    fn construct_solution_from_standard_model(
        &self,
        std_model: &StandardModel,
    ) -> SolverSolution<Var> {
        let std_solution = std_model.solution();

        if matches!(std_solution.status(), SolverStatus::Infeasible) {
            return SolverSolution::new_infeasible(
                *std_solution.iterations(),
                *std_solution.solve_time(),
            );
        }

        // 1. Map values back to original variables
        // We iterate over the original variables in the model and query the standard model for their values.
        let variable_values: HashMap<Var, f64> = self
            .variables
            .iter()
            .map(|var| (var.clone(), std_model.get_variable_value(var).unwrap())) // Added unwrap_or(0.0)
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

    pub fn variables(&self) -> &Vec<Var> {
        &self.variables
    }

    pub fn constraints(&self) -> &Vec<Constr> {
        &self.constraints
    }

    pub fn objective(&self) -> &Option<Objective> {
        &self.objective
    }

    pub fn solution(&self) -> &SolverSolution<Var> {
        &self.solution
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
        for constr in self.constraints.iter() {
            writeln!(f, "\t{},", constr)?;
        }
        writeln!(f, "]")?;

        // Display the variables
        writeln!(f, "Variables: [")?;
        for var in self.variables.iter() {
            writeln!(f, "\t{},", var)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}
