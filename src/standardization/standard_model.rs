use std::fmt;
use slotmap::{DenseSlotMap, SecondaryMap};

use crate::{
    core::{
        constraint::{Constraint, ConstraintSense},
        expression::LinearExpr,
        model::Model,
        objective::Objective,
        variable::{VariableKey, Variable, VariableType},
    },
    error::SolverError,
    simplex::{config::SolverConfig, solution::SolverSolution, solver::SimplexSolver},
    standardization::{
        standard_constraint::{StandardConstraint, StandardConstraintBuilder, StandardConstraintKey}, 
        standard_objective::StandardObjective,
        standard_variable::{StandardVariable, StandardVariableBuilder, StandardVariableKey},
    }
};

/// A model that enforces standard form constraints
#[derive(Debug, Default)]
pub struct StandardModel {
    variables: DenseSlotMap<StandardVariableKey, StandardVariable>,
    constraints: DenseSlotMap<StandardConstraintKey, StandardConstraint>,
    objective: Option<StandardObjective>,
    mapping: Option<VarMapping>,
    solution: SolverSolution<StandardVariableKey>,
    config: Option<SolverConfig>,
}

type VarMapping = SecondaryMap<VariableKey, (Option<StandardVariableKey>, Option<StandardVariableKey>)>;

impl StandardModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_model(model: &Model) -> Self {
        let mut standard_variables = DenseSlotMap::with_key();
        let mut mapping: VarMapping = SecondaryMap::new();

        // Step 1: Standardize variables and insert them into the arena immediately
        for (var_key, variable_data) in model.variables() {
            let (pos_data, neg_data) = Self::standardize_variable(variable_data);
            
            let pos_key = pos_data.map(|d| standard_variables.insert(d));
            let neg_key = neg_data.map(|d| standard_variables.insert(d));

            mapping.insert(var_key, (pos_key, neg_key));
        }

        // Step 2: Standardize constraints and add variables upper bound constraints
        let mut standard_constraints = DenseSlotMap::with_key();

        // 2a. Process constraints from the original model
        for constraint in model.constraints().values() {
            let std_constrs = Self::standardize_constraint(constraint, &standard_variables,  &mapping);
            for constr in std_constrs {
                standard_constraints.insert(constr);
            }
        }

        // 2b. Add upper bound constraints for variables
        // We iterate over the generated standard variables to check their bounds
        for (std_key, std_var) in standard_variables.iter() {
             let ub = std_var.upper_bound();
             if ub < f64::INFINITY {
                standard_constraints.insert(StandardConstraint::new(
                    LinearExpr::with_term(std_key, 1.0),
                    ub
                ));
            }
        }

        // Step 3: Transform and standardize objective
        let standard_objective = model
            .objective()
            .as_ref()
            .map(|objective| Self::standardize_objective(objective, &standard_variables, &mapping));

        Self {
            variables: standard_variables,
            constraints: standard_constraints,
            objective: standard_objective,
            mapping: Some(mapping),
            solution: SolverSolution::default(),
            config: None,
        }
    }

    pub fn with_config(mut self, config: SolverConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Add a new non-negative variable
    pub fn add_variable(&mut self) -> StandardVariableBuilder<'_> {
        StandardVariableBuilder::new(&mut self.variables)
    }

    /// Add a constraint in standard form: lhs ≤ rhs_constant
    pub fn add_constraint(&mut self, lhs: LinearExpr<StandardVariableKey>) -> StandardConstraintBuilder<'_> {
        StandardConstraintBuilder::new(&mut self.constraints, lhs)
    }

    /// Set the maximization objective
    pub fn set_objective(&mut self, expression: impl Into<LinearExpr<StandardVariableKey>>) {
        self.objective = Some(StandardObjective::new(expression.into()));
    }

    pub fn solve(&mut self) -> Result<(), SolverError> {
        if self.variables.is_empty() {
            return Err(SolverError::NoVariables);
        } else if self.objective.is_none() {
            return Err(SolverError::ObjectiveMissing);
        }

        let mut solver =
            SimplexSolver::form_standard_model(&self, self.config.clone().unwrap_or_default())?;
        self.solution = solver.start();
        Ok(())
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

    pub fn solution(&self) -> &SolverSolution<StandardVariableKey> {
        &self.solution
    }

    /// Helper to look up the value of an original variable from the standardized solution.
    pub fn get_variable_value(&self, var: VariableKey) -> Option<f64> {
        let mapping = self.mapping.as_ref()?;
        let solution_values = self.solution.variable_values().as_ref()?;
        let std_var = mapping.get(var)?;

        Some(match std_var {
            // Case: Split variable (x = x_pos - x_neg)
            (Some(pos), Some(neg)) => {
                let pos_value = solution_values.get(*pos).unwrap();
                let pos_shift = self.variables.get(*pos).unwrap().shift();

                let neg_value = solution_values.get(*neg).unwrap();
                let neg_shift = self.variables.get(*neg).unwrap().shift();
                
                (pos_value + pos_shift) - (neg_value + neg_shift)
            }
            // Case: Positive only (x = x_pos + shift)
            (Some(pos), None) => {
                let pos_shift = self.variables.get(*pos).unwrap().shift();
                solution_values.get(*pos).unwrap() + pos_shift
            }
            // Case: Negative only (x = -x_neg + shift)
            (None, Some(neg)) => {
                let neg_shift = self.variables.get(*neg).unwrap().shift();
                -solution_values.get(*neg).unwrap() + neg_shift
            }
            // Case: Variable optimized out or not found
            _ => 0.0,
        })
    }

    /// Standardize a variable into standard form (non-negative variables)
    fn standardize_variable(var: &Variable) -> (Option<StandardVariable>, Option<StandardVariable>) {
        // let std_var_name = format!("FromVar: {}", var.name_or_default());
        let std_var_name = format!("FromVariable: {}", var.name());

        match var.var_type() {
            VariableType::Binary => (
                // Binary variables are converted to a non-negative variable with upper bound of 1
                Some(
                    StandardVariable::new_positive()
                        .with_name(std_var_name)
                        .with_upper_bound(1.0),
                ),
                None,
            ),
            VariableType::Integer | VariableType::Continuous => {
                let lb = var.lower_bound();
                let ub = var.upper_bound();

                match (lb, ub) {
                    // Case 1: Lower bound is 0, create non-negative variable with optional upper bound
                    (0.0, _) => (
                        Some(
                            StandardVariable::new_positive()
                                .with_name(std_var_name)
                                .with_upper_bound(ub),
                        ),
                        None,
                    ),
                    // Case 2: Upper bound is 0, create non-positive variable
                    (_, 0.0) => (
                        None,
                        Some(
                            StandardVariable::new_negative()
                                .with_name(std_var_name)
                                .with_upper_bound(-lb),
                        ),
                    ),
                    // Case 3: Unbounded variable, split into positive and negative parts
                    (f64::NEG_INFINITY, f64::INFINITY) => (
                        Some(StandardVariable::new_positive().with_name(std_var_name.clone())),
                        Some(StandardVariable::new_negative().with_name(std_var_name)),
                    ),
                    // Case 4: Lower bound is negative infinity, create shifted negative variable
                    (f64::NEG_INFINITY, _) => (
                        None,
                        Some(
                            StandardVariable::new_negative()
                                .with_name(std_var_name)
                                .with_shift(ub),
                        ),
                    ),
                    // Case 5: Upper bound is infinity, create shifted positive variable
                    (_, f64::INFINITY) => (
                        Some(
                            StandardVariable::new_positive()
                                .with_name(std_var_name)
                                .with_shift(lb),
                        ),
                        None,
                    ),
                    // Case 6: Bounded variable within finite range, create shifted positive variable
                    _ => (
                        Some(
                            StandardVariable::new_positive()
                                .with_name(std_var_name)
                                .with_shift(lb)
                                .with_upper_bound(ub - lb),
                        ),
                        None,
                    ),
                }
            }
        }
    }

    /// Standardize a single constraint into standard form (ax ≤ b)
    fn standardize_constraint(
        constr: &Constraint,
        variables: &DenseSlotMap<StandardVariableKey, StandardVariable>,
        mapping: &VarMapping
    ) -> Vec<StandardConstraint> {
        let std_constr_name = format!("FromConstraint: {}", constr.name());
        // Move everything to LHS, constant to RHS
        let mut std_lhs = Self::standardize_expression(
            &(constr.lhs().clone() - constr.rhs().clone()),
            variables,
            mapping,
        );
        let std_rhs = -std_lhs.constant;
        std_lhs.constant = 0.0;

        match constr.sense() {
            ConstraintSense::LessEqual => {
                // Already in correct form
                vec![StandardConstraint::new(std_lhs, std_rhs).with_name(std_constr_name)]
            }
            ConstraintSense::GreaterEqual => {
                // Multiply by -1 to convert to ≤
                vec![StandardConstraint::new(-std_lhs, -std_rhs).with_name(std_constr_name)]
            }
            ConstraintSense::Equal => {
                // Split into x ≤ b and -x ≤ -b
                vec![
                    StandardConstraint::new(std_lhs.clone(), std_rhs).with_name(std_constr_name.clone()),
                    StandardConstraint::new(-std_lhs, -std_rhs).with_name(std_constr_name),
                ]
            }
        }
    }

    /// Standardize an objective into maximization form
    fn standardize_objective(
        obj: &Objective,
        variables: &DenseSlotMap<StandardVariableKey, StandardVariable>,
        mapping: &VarMapping
    ) -> StandardObjective {
        StandardObjective::from_sense(
            obj.sense(),
            Self::standardize_expression(obj.expr(), variables, mapping),
        )
    }

    /// Standardize a linear expression into standard form
    fn standardize_expression(
        expression: &LinearExpr<VariableKey>,
        variables: &DenseSlotMap<StandardVariableKey, StandardVariable>,
        mapping: &VarMapping,
    ) -> LinearExpr<StandardVariableKey> {
        let mut new_expr = LinearExpr::new();
        let mut shift = 0.0;

        for (var, coefficient) in &expression.terms {
            match mapping.get(*var).unwrap() {
                (Some(pos_var), Some(neg_var)) => {
                    let pos_shift = variables.get(*pos_var).unwrap().shift();
                    let neg_shift = variables.get(*neg_var).unwrap().shift();

                    shift += coefficient * pos_shift + coefficient * neg_shift;

                    new_expr.add_term(pos_var.clone(), *coefficient);
                    new_expr.add_term(neg_var.clone(), -coefficient);
                }
                (Some(pos_var), None) => {
                    let pos_shift = variables.get(*pos_var).unwrap().shift();
                    shift += coefficient * pos_shift;

                    new_expr.add_term(pos_var.clone(), *coefficient);
                }
                (None, Some(neg_var)) => {
                    let neg_shift = variables.get(*neg_var).unwrap().shift();
                    shift += coefficient * neg_shift;

                    new_expr.add_term(neg_var.clone(), -coefficient);
                }
                _ => {} // Ignore if no variable exists
            }
        }

        new_expr.add_constant(expression.constant + shift);
        new_expr
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
