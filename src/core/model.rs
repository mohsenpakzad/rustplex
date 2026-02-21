use std::fmt;
use slotmap::DenseSlotMap;

use crate::{
    core::{
        constraint::{Constraint, ConstraintBuilder, ConstraintKey},
        expression::LinearExpr,
        objective::{Objective, ObjectiveSense},
        variable::{Variable, VariableBuilder, VariableKey, VariableType},
    },
    error::SolverError,
    simplex::{config::SolverConfiguration, solution::SolverSolution},
    standard::standardizer::Standardizer,
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

    pub fn solve(&mut self) -> Result<(), SolverError> {
        if !self.is_lp() {
            return Err(SolverError::NonLinearNotSupported);
        } else if self.variables.is_empty() {
            return Err(SolverError::NoVariables);
        } else if self.objective.is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        // 1. Compile the domain model into a standard model
        let (standardizer, mut standardized_model) = Standardizer::compile(&self);

        println!("MODEL: {}", standardized_model);

        // 2. Solve the math
        standardized_model.solve()?;

        // 3. Lift the result back to the domain
        self.solution = standardizer.reconstruct_solution(standardized_model.solution(), &self);

        Ok(())
    }

    pub fn variables(&self) -> &DenseSlotMap<VariableKey, Variable> {
        &self.variables
    }

    pub fn constraints(&self) -> &DenseSlotMap<ConstraintKey, Constraint> {
        &self.constraints
    }

    pub fn objective(&self) -> Option<&Objective> {
        self.objective.as_ref()
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
