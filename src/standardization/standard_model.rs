use std::{collections::HashMap, fmt};

use crate::core::{
    constraint::{ConstrRef, ConstraintSense},
    expression::LinearExpr,
    model::Model,
    objective::Objective,
    variable::{VarRef, VariableType},
};

use super::{
    standard_constraint::StdConstrRef, standard_objective::StandardObjective,
    standard_variable::StdVarRef,
};

/// A model that enforces standard form constraints
#[derive(Debug)]
pub struct StandardModel {
    variables: Vec<StdVarRef>,
    constraints: Vec<StdConstrRef>,
    objective: Option<StandardObjective>,
    variable_map: Option<VariableMap>,
}

type VariableMap = HashMap<VarRef, (Option<StdVarRef>, Option<StdVarRef>)>;

impl StandardModel {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            constraints: Vec::new(),
            objective: None,
            variable_map: None,
        }
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
                let ub = std_var.get_upper_bound();
                if ub < f64::INFINITY {
                    Some(StdConstrRef::new(std_var.clone(), ub))
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
        }
    }

    /// Add a new non-negative variable
    pub fn add_variable(&mut self) -> StdVarRef {
        let std_var = StdVarRef::new();
        self.variables.push(std_var.clone());
        std_var
    }

    /// Add a constraint in standard form: lhs ≤ rhs_constant
    pub fn add_constraint(&mut self, lhs: LinearExpr<StdVarRef>, rhs: f64) -> StdConstrRef {
        let std_constr = StdConstrRef::new(lhs, rhs);
        self.constraints.push(std_constr.clone());
        std_constr
    }

    /// Set the maximization objective
    pub fn set_objective(&mut self, expression: LinearExpr<StdVarRef>) {
        self.objective = Some(StandardObjective::new(expression));
    }

    pub fn get_variables(&self) -> &Vec<StdVarRef> {
        &self.variables
    }

    pub fn get_constraints(&self) -> &Vec<StdConstrRef> {
        &self.constraints
    }

    pub fn get_objective(&self) -> &Option<StandardObjective> {
        &self.objective
    }

    /// Standardize a variable into standard form (non-negative variables)
    fn standardize_variable(var: &VarRef) -> (Option<StdVarRef>, Option<StdVarRef>) {
        let std_var_name = format!("FromVar: {}", var.get_name_or_default());

        match var.get_type() {
            VariableType::Binary => {
                // Binary variables are already standardized (0 ≤ x ≤ 1)
                (
                    Some(
                        StdVarRef::new_positive()
                            .name(std_var_name)
                            .upper_bound(1.0),
                    ),
                    None,
                )
            }
            VariableType::Integer | VariableType::Continuous => {
                let lb = var.get_lower_bound();
                let ub = var.get_upper_bound();

                if lb >= 0.0 {
                    // Already non-negative
                    (
                        Some(StdVarRef::new_positive().name(std_var_name).upper_bound(ub)),
                        None,
                    )
                } else if ub <= 0.0 {
                    (
                        None,
                        Some(
                            StdVarRef::new_negative()
                                .name(std_var_name)
                                .upper_bound(-lb),
                        ),
                    )
                } else if lb > f64::NEG_INFINITY && ub < f64::INFINITY {
                    // Bounded variable: shift to make non-negative
                    // x' = x - lb where x' ≥ 0
                    (
                        Some(
                            StdVarRef::new_negative()
                                .name(std_var_name)
                                .shift(lb)
                                .upper_bound(ub - lb),
                        ),
                        None,
                    )
                } else if lb == f64::NEG_INFINITY && ub == f64::INFINITY {
                    // Unrestricted variable: split into difference of two non-negative variables
                    // x = x⁺ - x⁻ where x⁺, x⁻ ≥ 0
                    (
                        Some(StdVarRef::new_positive().name(std_var_name.clone())),
                        Some(StdVarRef::new_negative().name(std_var_name)),
                    )
                } else if lb == f64::NEG_INFINITY {
                    // Lower-unbounded: split and apply upper bound to positive part
                    // x = x⁺ - x⁻ where x⁺ ≥ 0, x⁻ ≤ ub
                    (
                        Some(
                            StdVarRef::new_positive()
                                .name(std_var_name.clone())
                                .upper_bound(ub),
                        ),
                        Some(StdVarRef::new_negative().name(std_var_name)),
                    )
                } else {
                    // ub == f64::INFINITY
                    // Upper-unbounded: split and apply lower bound to negative part
                    // x = x⁺ - x⁻ where x⁺ ≤ ub, x⁻ ≥ 0
                    (
                        Some(StdVarRef::new_positive().name(std_var_name.clone())),
                        Some(
                            StdVarRef::new_negative()
                                .name(std_var_name)
                                .upper_bound(-lb),
                        ),
                    )
                }
            }
        }
    }

    /// Standardize a single constraint into standard form (ax ≤ b)
    fn standardize_constraint(constr: &ConstrRef, variable_map: &VariableMap) -> Vec<StdConstrRef> {
        let std_constr_name = format!("FromConstr: {}", constr.get_name_or_default());
        let lhs = constr.get_lhs();
        let rhs = constr.get_rhs();
        // Move everything to LHS, constant to RHS
        let mut std_lhs = Self::standardize_expression(&(lhs.clone() - rhs.clone()), variable_map);
        std_lhs.constant = 0.0;
        let std_rhs = rhs.constant - lhs.constant;

        match constr.get_sense() {
            ConstraintSense::LessEqual => {
                // Already in correct form
                vec![StdConstrRef::new(std_lhs, std_rhs).name(std_constr_name)]
            }
            ConstraintSense::GreaterEqual => {
                // Multiply by -1 to convert to ≤
                vec![StdConstrRef::new(-std_lhs, -std_rhs).name(std_constr_name)]
            }
            ConstraintSense::Equal => {
                // Split into x ≤ b and -x ≤ -b
                vec![
                    StdConstrRef::new(std_lhs.clone(), std_rhs).name(std_constr_name.clone()),
                    StdConstrRef::new(-std_lhs, -std_rhs).name(std_constr_name),
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

    fn standardize_expression(
        expression: &LinearExpr<VarRef>,
        variable_map: &VariableMap,
    ) -> LinearExpr<StdVarRef> {
        let std_terms = expression
            .terms
            .iter()
            .flat_map(|(var, &coefficient)| match variable_map.get(var).unwrap() {
                (Some(pos_var), Some(neg_var)) => vec![
                    (pos_var.clone(), coefficient),
                    (neg_var.clone(), -coefficient),
                ],
                (Some(pos_var), None) => vec![(pos_var.clone(), coefficient)],
                (None, Some(neg_var)) => vec![(neg_var.clone(), -coefficient)],
                _ => vec![],
            })
            .collect::<HashMap<StdVarRef, f64>>();

        LinearExpr::with_terms_and_constant(std_terms, expression.constant)
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
