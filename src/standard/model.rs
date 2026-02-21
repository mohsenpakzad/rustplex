use std::fmt;
use slotmap::DenseSlotMap;

use crate::{
    core::expression::LinearExpr,
    error::SolverError,
    simplex::{
        config::SolverConfiguration,
        solution::SolverSolution,
        solver::SimplexSolver
    },
    standard::{
        constraint::{StandardConstraint, StandardConstraintBuilder, StandardConstraintKey},
        objective::StandardObjective,
        variable::{StandardVariable, StandardVariableBuilder, StandardVariableKey}
    }
};

/// A model that enforces standard form constraints
#[derive(Debug)]
pub struct StandardModel {
    variables: DenseSlotMap<StandardVariableKey, StandardVariable>,
    constraints: DenseSlotMap<StandardConstraintKey, StandardConstraint>,
    objective: Option<StandardObjective>,
    config: SolverConfiguration,
}

impl StandardModel {
    pub fn new() -> Self {
        Self {
            variables: DenseSlotMap::with_key(),
            constraints: DenseSlotMap::with_key(),
            objective: None,
            config: SolverConfiguration::default(),
        }
    }

    pub fn with_config(mut self, config: SolverConfiguration) -> Self {
        self.config = config;
        self
    }

    /// Add a new non-negative variable
    pub fn build_variable(&mut self) -> StandardVariableBuilder<'_> {
        StandardVariableBuilder::new(&mut self.variables)
    }

    /// Add a constraint in standard form: lhs ≤ rhs_constant
    pub fn build_constraint(&mut self, lhs: LinearExpr<StandardVariableKey>) -> StandardConstraintBuilder<'_> {
        StandardConstraintBuilder::new(&mut self.constraints, lhs)
    }

    pub fn add_variable(&mut self, var: StandardVariable) -> StandardVariableKey {
        self.variables.insert(var)
    }

    pub fn add_constraint(&mut self, constr: StandardConstraint) -> StandardConstraintKey {
        self.constraints.insert(constr)
    }

    /// Set the maximization objective
    pub fn set_objective(&mut self, expression: impl Into<LinearExpr<StandardVariableKey>>) {
        self.objective = Some(StandardObjective::new(expression.into()));
    }

    pub fn solve(&mut self) -> Result<SolverSolution<StandardVariableKey>, SolverError> {
        if self.variables.is_empty() {
            return Err(SolverError::NoVariables);
        } else if self.objective.is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        let mut solver = SimplexSolver::form_standard_model(&self, self.config)?;

        let solution = solver.start();

        Ok(solution)
    }

    pub fn variables(&self) -> &DenseSlotMap<StandardVariableKey, StandardVariable> {
        &self.variables
    }

    pub fn constraints(&self) -> &DenseSlotMap<StandardConstraintKey, StandardConstraint> {
        &self.constraints
    }

    pub fn objective(&self) -> &Option<StandardObjective> {
        &self.objective
    }
}

impl fmt::Display for StandardModel {
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
