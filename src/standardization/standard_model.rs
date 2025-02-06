use std::{collections::HashMap, fmt};

use crate::{
    core::{
        constraint::{Constr, ConstraintSense},
        expression::LinearExpr,
        model::Model,
        objective::Objective,
        variable::{Var, VariableType},
    },
    simplex::{config::SolverConfig, solution::SolverSolution, solver::SimplexSolver},
};

use super::{
    standard_constraint::StdConstr, standard_objective::StandardObjective,
    standard_variable::StdVar,
};

/// A model that enforces standard form constraints
#[derive(Debug, Default)]
pub struct StandardModel {
    variables: Vec<StdVar>,
    constraints: Vec<StdConstr>,
    objective: Option<StandardObjective>,
    variable_map: Option<VariableMap>,
    solution: SolverSolution<StdVar>,
    config: Option<SolverConfig>,
}

type VariableMap = HashMap<Var, (Option<StdVar>, Option<StdVar>)>;

impl StandardModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_model(model: &Model) -> Self {
        // Step 1: Create a mapping of original variables to their standardized form
        let variable_map = model
            .get_variables()
            .iter()
            .map(|var| (var.clone(), (Self::standardize_variable(var))))
            .collect::<VariableMap>();

        // Step 2: Standardize variables
        let standard_variables = variable_map
            .values()
            .flat_map(|std_var| match std_var {
                (Some(pos_var), Some(neg_var)) => vec![pos_var.clone(), neg_var.clone()],
                (Some(pos_var), None) => vec![pos_var.clone()],
                (None, Some(neg_var)) => vec![neg_var.clone()],
                _ => vec![],
            })
            .collect::<Vec<_>>();

        // Step 3: Standardize constraints and add variables upper bound constraints
        let standard_constraints: Vec<_> = model
            .get_constraints()
            .iter()
            .flat_map(|constraint| Self::standardize_constraint(constraint, &variable_map))
            .chain(standard_variables.iter().filter_map(|std_var| {
                // Create upper bound constraints for variables
                let ub = std_var.get_upper_bound();
                if ub < f64::INFINITY {
                    Some(StdConstr::new(
                        LinearExpr::with_term(std_var.clone(), 1.0),
                        ub,
                    ))
                } else {
                    None
                }
            }))
            .collect();

        // Step 4: Transform and standardize objective
        let standard_objective = model
            .get_objective()
            .as_ref()
            .map(|objective| Self::standardize_objective(&objective, &variable_map));

        Self {
            variables: standard_variables,
            constraints: standard_constraints,
            objective: standard_objective,
            variable_map: Some(variable_map),
            solution: SolverSolution::default(),
            config: None,
        }
    }

    pub fn with_config(mut self, config: SolverConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Add a new non-negative variable
    pub fn add_variable(&mut self) -> StdVar {
        let std_var = StdVar::new();
        self.variables.push(std_var.clone());
        std_var
    }

    /// Add a constraint in standard form: lhs ≤ rhs_constant
    pub fn add_constraint(&mut self, lhs: impl Into<LinearExpr<StdVar>>, rhs: f64) -> StdConstr {
        let std_constr = StdConstr::new(lhs.into(), rhs);
        self.constraints.push(std_constr.clone());
        std_constr
    }

    /// Set the maximization objective
    pub fn set_objective(&mut self, expression: impl Into<LinearExpr<StdVar>>) {
        self.objective = Some(StandardObjective::new(expression.into()));
    }

    pub fn solve(&mut self) {
        let mut solver =
            SimplexSolver::form_standard_model(&self, self.config.clone().unwrap_or_default());
        self.solution = solver.start();
    }

    pub fn get_variables(&self) -> &Vec<StdVar> {
        &self.variables
    }

    pub fn get_constraints(&self) -> &Vec<StdConstr> {
        &self.constraints
    }

    pub fn get_objective(&self) -> &Option<StandardObjective> {
        &self.objective
    }

    pub fn get_solution(&self) -> &SolverSolution<StdVar> {
        &self.solution
    }

    pub fn get_model_solution(&self) -> Option<SolverSolution<Var>> {
        let variable_map = self.variable_map.as_ref()?;

        let solution_values = self.solution.get_variable_values();
        if solution_values.is_none() {
            return Some(self.solution.clone_with_new_variable_type(None));
        }
        let solution_values = solution_values.as_ref().unwrap();

        let mapped_values = variable_map
            .iter()
            .map(|(var, std_var)| {
                let value = match std_var {
                    (Some(pos), Some(neg)) => {
                        let pos_value = solution_values.get(pos).unwrap() + pos.get_shift();
                        let neg_value = solution_values.get(neg).unwrap() + neg.get_shift();
                        pos_value - neg_value
                    }
                    (Some(pos), None) => solution_values.get(pos).unwrap() + pos.get_shift(),
                    (None, Some(neg)) => -(solution_values.get(neg).unwrap() + neg.get_shift()),
                    _ => 0.0,
                };
                (var.clone(), value)
            })
            .collect();

        Some(
            self.solution
                .clone_with_new_variable_type(Some(mapped_values)),
        )
    }

    /// Standardize a variable into standard form (non-negative variables)
    fn standardize_variable(var: &Var) -> (Option<StdVar>, Option<StdVar>) {
        let std_var_name = format!("FromVar: {}", var.get_name_or_default());

        match var.get_type() {
            VariableType::Binary => (
                // Binary variables are converted to a non-negative variable with upper bound of 1
                Some(StdVar::new_positive().name(std_var_name).upper_bound(1.0)),
                None,
            ),
            VariableType::Integer | VariableType::Continuous => {
                let lb = var.get_lower_bound();
                let ub = var.get_upper_bound();

                match (lb, ub) {
                    // Case 1: Lower bound is 0, create non-negative variable with optional upper bound
                    (0.0, _) => (
                        Some(StdVar::new_positive().name(std_var_name).upper_bound(ub)),
                        None,
                    ),
                    // Case 2: Upper bound is 0, create non-positive variable
                    (_, 0.0) => (
                        None,
                        Some(StdVar::new_negative().name(std_var_name).upper_bound(-lb)),
                    ),
                    // Case 3: Unbounded variable, split into positive and negative parts
                    (f64::NEG_INFINITY, f64::INFINITY) => (
                        Some(StdVar::new_positive().name(std_var_name.clone())),
                        Some(StdVar::new_negative().name(std_var_name)),
                    ),
                    // Case 4: Lower bound is negative infinity, create shifted negative variable
                    (f64::NEG_INFINITY, _) => (
                        None,
                        Some(StdVar::new_negative().name(std_var_name).shift(ub)),
                    ),
                    // Case 5: Upper bound is infinity, create shifted positive variable
                    (_, f64::INFINITY) => (
                        Some(StdVar::new_positive().name(std_var_name).shift(lb)),
                        None,
                    ),
                    // Case 6: Bounded variable within finite range, create shifted positive variable
                    _ => (
                        Some(
                            StdVar::new_positive()
                                .name(std_var_name)
                                .shift(lb)
                                .upper_bound(ub - lb),
                        ),
                        None,
                    ),
                }
            }
        }
    }

    /// Standardize a single constraint into standard form (ax ≤ b)
    fn standardize_constraint(constr: &Constr, variable_map: &VariableMap) -> Vec<StdConstr> {
        let std_constr_name = format!("FromConstr: {}", constr.get_name_or_default());
        // Move everything to LHS, constant to RHS
        let mut std_lhs = Self::standardize_expression(
            &(constr.get_lhs().clone() - constr.get_rhs().clone()),
            variable_map,
        );
        let std_rhs = -std_lhs.constant;
        std_lhs.constant = 0.0;

        match constr.get_sense() {
            ConstraintSense::LessEqual => {
                // Already in correct form
                vec![StdConstr::new(std_lhs, std_rhs).name(std_constr_name)]
            }
            ConstraintSense::GreaterEqual => {
                // Multiply by -1 to convert to ≤
                vec![StdConstr::new(-std_lhs, -std_rhs).name(std_constr_name)]
            }
            ConstraintSense::Equal => {
                // Split into x ≤ b and -x ≤ -b
                vec![
                    StdConstr::new(std_lhs.clone(), std_rhs).name(std_constr_name.clone()),
                    StdConstr::new(-std_lhs, -std_rhs).name(std_constr_name),
                ]
            }
        }
    }

    /// Standardize an objective into maximization form
    fn standardize_objective(obj: &Objective, variable_map: &VariableMap) -> StandardObjective {
        StandardObjective::from_sense(
            obj.get_sense(),
            Self::standardize_expression(obj.get_expr(), variable_map),
        )
    }

    /// Standardize a linear expression into standard form
    fn standardize_expression(
        expression: &LinearExpr<Var>,
        variable_map: &VariableMap,
    ) -> LinearExpr<StdVar> {
        let mut std_terms = HashMap::new();
        let mut shift = 0.0;

        for (var, &coefficient) in &expression.terms {
            match variable_map.get(var).unwrap() {
                (Some(pos_var), Some(neg_var)) => {
                    shift += coefficient * pos_var.get_shift() + coefficient * neg_var.get_shift();

                    std_terms.insert(pos_var.clone(), coefficient);
                    std_terms.insert(neg_var.clone(), -coefficient);
                }
                (Some(pos_var), None) => {
                    shift += coefficient * pos_var.get_shift();

                    std_terms.insert(pos_var.clone(), coefficient);
                }
                (None, Some(neg_var)) => {
                    shift += coefficient * neg_var.get_shift();

                    std_terms.insert(neg_var.clone(), -coefficient);
                }
                _ => {} // Ignore if no variable exists
            }
        }

        LinearExpr::with_terms_and_constant(std_terms, expression.constant + shift)
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
