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
    config: SolverConfiguration,
}

pub struct ModelDisplay<'a, T> {
    pub model: &'a Model,
    pub item: T,
}

impl Model {
    /// Creates a new, empty model with default settings.
    pub fn new() -> Self {
        Self {
            variables: DenseSlotMap::with_key(),
            constraints: DenseSlotMap::with_key(),
            objective: None,
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

    /// Returns a helper object that implements Display, allowing you to print
    /// keys (Variables/Constraints) using their actual Names from the Model.
    pub fn format<'a, T>(&'a self, item: T) -> ModelDisplay<'a, T> {
        ModelDisplay { model: self, item }
    }

    fn is_lp(&self) -> bool {
        !self
            .variables
            .values()
            .any(|variable| !matches!(variable.var_type(), VariableType::Continuous))
    }

    pub fn solve(&mut self) -> Result<SolverSolution<VariableKey>, SolverError> {
        if !self.is_lp() {
            return Err(SolverError::NonLinearNotSupported);
        } else if self.variables.is_empty() {
            return Err(SolverError::NoVariables);
        } else if self.objective.is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        // 1. Compile the domain model into a standard model
        let (standardizer, mut standardized_model) = Standardizer::compile(&self);

        // 2. Solve the math
        let std_solution = standardized_model.solve()?;

        // 3. Lift the result back to the domain
        let solution = standardizer.reconstruct_solution(&std_solution, &self);

        Ok(solution)
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

impl<'a> fmt::Display for ModelDisplay<'a, VariableKey> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.model.variables().get(self.item).unwrap())
    }
}

impl<'a> fmt::Display for ModelDisplay<'a, ConstraintKey> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.model.constraints().get(self.item).unwrap())
    }
}

impl<'a> fmt::Display for ModelDisplay<'a, &LinearExpr<VariableKey>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for (var, coeff) in &self.item.terms {
            if *coeff == 0.0 { continue; }
            
            if !first {
                write!(f, " {} ", if *coeff > 0.0 { "+" } else { "-" })?;
            } else if *coeff < 0.0 {
                write!(f, "-")?;
            }

            let abs_coeff = coeff.abs();
            if (abs_coeff - 1.0).abs() > 1e-10 {
                write!(f, "{:.2} * ", abs_coeff)?;
            }

            // RECURSIVE MAGIC: Use model.format to print the variable name
            write!(f, "{}", self.model.format(*var))?;
            first = false;
        }
        if first { write!(f, "0")?; } // Handle empty/zero expression
        // Add constant if exists
        if self.item.constant.abs() > 1e-10 {
             write!(f, " + {:.2}", self.item.constant)?;
        }
        Ok(())
    }
}

impl<'a> fmt::Display for ModelDisplay<'a, &SolverSolution<VariableKey>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Solver Status: {:?}", self.item.status())?;
        if self.item.status().is_optimal() {
            writeln!(
                f,
                "Objective Value: {:.2}",
                self.item.objective_value().unwrap_or(0.0)
            )?;
        } else {
            writeln!(f, "Objective Value: {:?}", self.item.objective_value())?;
        }

        if let Some(vars) = self.item.variable_values() {
            writeln!(f, "Variable Values: [")?;
            for (var_key, value) in vars {
                writeln!(f, "\t{}: {:.2}", self.model.variables.get(var_key).unwrap(), value)?;
            }
            writeln!(f, "]")?;
        } else {
            writeln!(f, "Variable Values: None")?;
        }
        writeln!(f, "Iterations: {}", self.item.iterations())?;
        write!(f, "Solve Time: {:.2?}", self.item.solve_time())?;
        Ok(())
    }
}