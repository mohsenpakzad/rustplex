use std::{collections::HashMap, fmt, mem};

use crate::{
    core::expression::LinearExpr,
    standardization::{standard_model::StandardModel, standard_variable::StdVar},
};

use super::{dict_entry::DictEntryRef, dict_variable::DictVar};

#[derive(Debug, Clone)]
pub struct SlackDictionary {
    objective: LinearExpr<DictVar>,
    entries: Vec<DictEntryRef>,
    variable_map: HashMap<StdVar, DictVar>,
}

impl SlackDictionary {
    pub fn from_standard_model(standard_model: &StandardModel) -> Self {
        let variable_map = standard_model
            .variables()
            .iter()
            .map(|var| (var.clone(), DictVar::new_non_slack(var.clone())))
            .collect::<HashMap<_, _>>();

        let entries = standard_model
            .constraints()
            .iter()
            .enumerate()
            .map(|(idx, constr)| {
                DictEntryRef::new(
                    DictVar::new_slack(idx),
                    Self::transform_expression(&(constr.rhs() - constr.lhs()), &variable_map),
                )
            })
            .collect::<Vec<_>>();

        let objective = standard_model
            .objective()
            .as_ref()
            .map(|obj| Self::transform_expression(obj.expr(), &variable_map))
            .unwrap();

        Self {
            objective,
            entries,
            variable_map,
        }
    }

    pub fn set_objective(&mut self, objective: LinearExpr<DictVar>) {
        self.objective = objective;
    }

    pub fn replace_objective(&mut self, new_objective: LinearExpr<DictVar>) -> LinearExpr<DictVar> {
        mem::replace(&mut self.objective, new_objective)
    }

    pub fn objective(&self) -> &LinearExpr<DictVar> {
        &self.objective
    }

    pub fn entries(&self) -> &Vec<DictEntryRef> {
        &self.entries
    }

    pub fn variable_map(&self) -> &HashMap<StdVar, DictVar> {
        &self.variable_map
    }

    pub fn objective_value(&self) -> f64 {
        self.objective.constant
    }

    pub fn basic_values(&self) -> HashMap<DictVar, f64> {
        self.entries
            .iter()
            .map(|entry| (entry.basic_var().clone(), entry.value()))
            .collect()
    }

    pub fn std_values(&self) -> HashMap<StdVar, f64> {
        let basic_to_entry = self
            .entries
            .iter()
            .map(|entry| (entry.basic_var(), entry.clone()))
            .collect::<HashMap<_, _>>();

        self.variable_map
            .iter()
            .map(|(std_var, dict_var)| {
                (
                    std_var.clone(),
                    basic_to_entry
                        .get(dict_var)
                        .map(DictEntryRef::value)
                        .unwrap_or(0.0),
                )
            })
            .collect()
    }

    pub fn add_var_to_all_entries(&mut self, var: DictVar, coefficient: f64) {
        for entry in self.entries.iter_mut() {
            entry.add_non_basic(var.clone(), coefficient);
        }
    }

    pub fn remove_var_from_all_entries(&mut self, var: DictVar) {
        for entry in self.entries.iter_mut() {
            entry.remove_non_basic(var.clone());
        }
    }

    pub fn remove_entry_at(&mut self, index: usize) {
        self.entries.remove(index);
    }

    pub fn pivot(&mut self, entering: &DictVar, leaving: &DictEntryRef) {
        leaving.switch_to_basic(entering.clone());
        let leaving_expr = leaving.expr();

        // Iterate over ALL entries.
        // If an entry contains the 'entering' variable, the method below will swap it.
        // If it doesn't contain it, the method returns None and does nothing.
        for entry in self.entries.iter() {
            // We don't need to substitute in the pivot row itself
            // (variable was already removed by switch_to_basic), but explicitly skipping it is cleaner.
            // We compare basic variables to identify if it's the same row.
            if entry.basic_var() != leaving.basic_var() {
                entry.replace_non_basic_with_expr(entering.clone(), &leaving_expr);
            }
        }

        // Update objective
        self.objective
            .replace_var_with_expr(entering.clone(), &leaving_expr);
    }

    fn transform_expression(
        expression: &LinearExpr<StdVar>,
        variable_map: &HashMap<StdVar, DictVar>,
    ) -> LinearExpr<DictVar> {
        let std_terms = expression
            .terms
            .iter()
            .map(|(var, coefficient)| (variable_map.get(var).unwrap().clone(), *coefficient))
            .collect::<Vec<(DictVar, f64)>>();

        LinearExpr::with_terms_and_constant(std_terms, expression.constant)
    }
}

impl fmt::Display for SlackDictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the objective
        writeln!(f, "Objective = {}", self.objective)?;

        // Display the entires
        for entry in self.entries.iter() {
            writeln!(f, "{}", entry)?;
        }
        Ok(())
    }
}
