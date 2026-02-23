use slotmap::SecondaryMap;

use crate::{
    common::expression::LinearExpr,
    modeling::{
        constraint::{Constraint, ConstraintSense},
        model::Model,
        objective::{Objective, ObjectiveSense},
        variable::{Variable, VariableKey, VariableType}
    }, 
    solver::solution::SolverSolution,
    standard_form::{
        constraint::StandardConstraint,
        model::StandardModel,
        variable::{StandardVariable, StandardVariableKey}
    }
};

enum VariableMapping {
    Positive { pos_var: StandardVariableKey, shift: f64 },
    Negative { neg_var: StandardVariableKey, shift: f64 },
    Split { pos_var: StandardVariableKey, neg_var: StandardVariableKey },
}

pub struct Standardizer {
    mapping: SecondaryMap<VariableKey, VariableMapping>,
}

impl Standardizer {
    /// Compiles a user Model into a pure StandardModel and retains mapping info
    pub fn compile(model: &Model) -> (Self, StandardModel) {
        let mut std_model = StandardModel::new().with_config(*model.config());
        let mut mapping = SecondaryMap::new();

        // Step 1: Standardize variables
        model
            .variables()
            .iter()
            .map(|(var_key, var)| (var_key, Self::standardize_variable(var, &mut std_model)))
            .for_each(|(var_key, var_mapping)| {
                mapping.insert(var_key, var_mapping);
            });

        // Step 2: Standardize constraints
        model
            .constraints()
            .values()
            .for_each(|constr| {
                Self::standardize_constraint(constr, &mut std_model, &mapping)
            });

        // Step 3: standardize objective
        model
            .objective()
            .map(|objective| Self::standardize_objective(objective, &mut std_model, &mapping));

        (Self { mapping }, std_model)
    }

    /// Lifts the StandardModel solution back to the domain VariableKeys
    pub fn reconstruct_solution(
        &self, 
        std_solution: &SolverSolution<StandardVariableKey>,
        original_model: &Model,
    ) -> SolverSolution<VariableKey> {
        let std_values = match std_solution.variable_values() {
            Some(vals) => vals,
            None => return SolverSolution::new_infeasible(*std_solution.iterations(), *std_solution.solve_time()),
        };

        // 1. Handle Objective Value and Sign
        // If the objective was Minimize, we must negate the result (since Simplex solved for Max -Z)
        let objective_value = match original_model.objective().unwrap().sense() {
            ObjectiveSense::Maximize => std_solution.objective_value().unwrap(),
            ObjectiveSense::Minimize => -std_solution.objective_value().unwrap()
        };

        // 2. Map values back to original variables
        // We iterate over the original variables in the model and query the standard model for their values.
        let variable_values = self.mapping
            .iter()
            .map(|(var_key, var_mapping)| match var_mapping {
                // Case: Split variable (x = x_pos - x_neg)
                VariableMapping::Split { pos_var, neg_var } => {
                    (var_key, std_values.get(*pos_var).unwrap() - std_values.get(*neg_var).unwrap())
                },
                // Case: Positive only (x = x_pos + shift)
                VariableMapping::Positive { pos_var, shift } => {
                    (var_key, std_values.get(*pos_var).unwrap() + shift)
                },
                // Case: Negative only (x = -x_neg + shift)
                VariableMapping::Negative { neg_var, shift } => {
                    (var_key, -std_values.get(*neg_var).unwrap() + shift)
                }
            }).collect::<SecondaryMap<_,_>>();

        SolverSolution::new(
            *std_solution.status(),
            objective_value,
            variable_values,
            *std_solution.iterations(),
            *std_solution.solve_time(),
        )
    }

    // --- Private Compilation Helpers ---

    /// Standardize a variable into standard form (non-negative variables)
    fn standardize_variable(var: &Variable, std_model: &mut StandardModel) -> VariableMapping {
        let pos_var = || StandardVariable::new().with_name(format!("FromVariable: {}⁺", var.name()));
        let neg_var = || StandardVariable::new().with_name(format!("FromVariable: {}⁻", var.name()));

        match var.var_type() {
            // Binary variables are converted to a non-negative variable with upper bound of 1
            VariableType::Binary => {
                let pos_var = std_model.add_variable(pos_var());
                let shift = 0.0;
                let upper_bound = 1.0;

                std_model.add_constraint(StandardConstraint::new(pos_var, upper_bound));
                VariableMapping::Positive { pos_var, shift }
            },
            VariableType::Integer | VariableType::Continuous => {
                let lb = var.lower_bound();
                let ub = var.upper_bound();

                match (lb, ub) {
                    // Case 1: Lower bound is 0, create non-negative variable with optional upper bound
                    (0.0, _) => {
                        let pos_var = std_model.add_variable(pos_var());
                        let shift = 0.0;
                        let upper_bound = ub;

                        if upper_bound < f64::INFINITY {
                            std_model.add_constraint(StandardConstraint::new(pos_var, upper_bound));
                        }
                        VariableMapping::Positive { pos_var, shift }
                    },
                    // Case 2: Upper bound is 0, create non-positive variable
                    (_, 0.0) => {
                        let pos_var = std_model.add_variable(pos_var());
                        let shift = lb;
                        let upper_bound = -lb;

                        if upper_bound < f64::INFINITY {
                            std_model.add_constraint(StandardConstraint::new(pos_var, upper_bound));
                        }
                        VariableMapping::Positive { pos_var, shift }
                    },
                    // Case 3: Unbounded variable, split into positive and negative parts
                    (f64::NEG_INFINITY, f64::INFINITY) => {
                        let pos_var = std_model.add_variable(pos_var());
                        let neg_var = std_model.add_variable(neg_var());

                        VariableMapping::Split { pos_var, neg_var }
                    },
                    // Case 4: Lower bound is negative infinity, create shifted negative variable
                    (f64::NEG_INFINITY, _) => {
                        let neg_var = std_model.add_variable(neg_var());
                        let shift = ub;

                        VariableMapping::Negative { neg_var, shift }
                    },
                    // Case 5: Upper bound is infinity, create shifted positive variable
                    (_, f64::INFINITY) => {
                        let pos = std_model.add_variable(pos_var());
                        let shift = lb;

                        VariableMapping::Positive { pos_var: pos, shift }
                    },
                    // Case 6: Bounded variable within finite range, create shifted positive variable
                    _ => {
                        let pos_var = std_model.add_variable(pos_var());
                        let shift = lb;
                        let upper_bound = ub - lb;

                        if upper_bound < f64::INFINITY {
                            std_model.add_constraint(StandardConstraint::new(pos_var, upper_bound));
                        }
                        VariableMapping::Positive { pos_var, shift }
                    },
                }
            }
        }
    }

    /// Standardize a single constraint into standard form (ax ≤ b)
    fn standardize_constraint(
        constr: &Constraint,
        std_model: &mut StandardModel,
        mapping: &SecondaryMap<VariableKey, VariableMapping>,
    ) {
        let std_constr_name = format!("FromConstraint: {}", constr.name());
        // Move everything to LHS, constant to RHS
        let mut std_lhs = Self::standardize_expression(
            &(constr.lhs().clone() - constr.rhs().clone()),
            mapping,
        );
        let std_rhs = -std_lhs.constant;
        std_lhs.constant = 0.0;

        match constr.sense() {
            ConstraintSense::LessEqual => {
                // Already in correct form
                std_model.add_constraint(StandardConstraint::new(std_lhs, std_rhs).with_name(std_constr_name));
            }
            ConstraintSense::GreaterEqual => {
                // Multiply by -1 to convert to ≤
                std_model.add_constraint(StandardConstraint::new(-std_lhs, -std_rhs).with_name(std_constr_name));
            }
            ConstraintSense::Equal => {
                // Split into x ≤ b and -x ≤ -b
                std_model.add_constraint(StandardConstraint::new(std_lhs.clone(), std_rhs).with_name(std_constr_name.clone()));
                std_model.add_constraint(StandardConstraint::new(-std_lhs, -std_rhs).with_name(std_constr_name));
            }
        }
    }

    /// Standardize an objective into maximization form
    fn standardize_objective(
        obj: &Objective,
        std_model: &mut StandardModel,
        mapping: &SecondaryMap<VariableKey, VariableMapping>,
    ) {
        let std_expr = Self::standardize_expression(obj.expr(), mapping);
        std_model.set_objective(match obj.sense() {
                ObjectiveSense::Minimize => -std_expr,
                ObjectiveSense::Maximize => std_expr,
        });
    }

    /// Standardize a linear expression into standard form
    fn standardize_expression(
        expression: &LinearExpr<VariableKey>,
        mapping: &SecondaryMap<VariableKey, VariableMapping>,
    ) -> LinearExpr<StandardVariableKey> {
        let mut new_expr = LinearExpr::new();
        let mut expr_shift = 0.0;

        for (var_key, coefficient) in &expression.terms {
            match mapping.get(*var_key).unwrap() {
                VariableMapping::Split { pos_var, neg_var } => {
                    new_expr.add_term(pos_var.clone(), *coefficient);
                    new_expr.add_term(neg_var.clone(), -coefficient);
                }
                VariableMapping::Positive { pos_var, shift } => {
                    expr_shift += coefficient * shift;
                    new_expr.add_term(pos_var.clone(), *coefficient);
                }
                VariableMapping::Negative { neg_var, shift } => {
                    expr_shift += coefficient * shift;
                    new_expr.add_term(neg_var.clone(), -coefficient);
                }
            }
        }
        new_expr.add_constant(expression.constant + expr_shift);
        new_expr
    }
}