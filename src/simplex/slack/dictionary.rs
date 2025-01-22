use std::{collections::HashMap, fmt, mem};

use crate::{
    core::expression::LinearExpr,
    standardization::{standard_model::StandardModel, standard_variable::StdVarRef},
};

use super::{dict_entry::DictEntryRef, dict_variable::DictVarRef};

#[derive(Debug, Clone)]
pub struct SlackDictionary {
    objective: LinearExpr<DictVarRef>,
    entries: Vec<DictEntryRef>,
    variable_map: VariableMap,
}

type VariableMap = HashMap<StdVarRef, DictVarRef>;

impl SlackDictionary {
    pub fn from_standard_model(standard_model: &StandardModel) -> Self {
        let variable_map = standard_model
            .get_variables()
            .iter()
            .map(|var| (var.clone(), DictVarRef::new_non_slack(var.clone())))
            .collect::<VariableMap>();

        let entries = standard_model
            .get_constraints()
            .iter()
            .enumerate()
            .map(|(idx, constr)| {
                DictEntryRef::new(
                    DictVarRef::new_slack(idx),
                    Self::transform_expression(
                        &(constr.get_rhs() - constr.get_lhs()),
                        &variable_map,
                    ),
                )
            })
            .collect();

        let objective = standard_model
            .get_objective()
            .as_ref()
            .map(|obj| Self::transform_expression(obj.get_expr(), &variable_map))
            .unwrap();

        Self {
            objective,
            entries,
            variable_map,
        }
    }

    pub fn set_objective(&mut self, objective: LinearExpr<DictVarRef>) {
        self.objective = objective;
    }

    pub fn replace_objective(
        &mut self,
        new_objective: LinearExpr<DictVarRef>,
    ) -> LinearExpr<DictVarRef> {
        mem::replace(&mut self.objective, new_objective)
    }

    pub fn get_objective(&self) -> &LinearExpr<DictVarRef> {
        &self.objective
    }

    pub fn get_entires(&self) -> &Vec<DictEntryRef> {
        &self.entries
    }

    pub fn get_variable_map(&self) -> &VariableMap {
        &self.variable_map
    }

    pub fn add_var_to_all_entries(&mut self, var: DictVarRef, coefficient: f64) {
        for entry in self.entries.iter_mut() {
            entry.add_non_basic(var.clone(), coefficient)
        }
    }

    pub fn remove_var_from_all_entries(&mut self, var: DictVarRef) {
        for entry in self.entries.iter_mut() {
            entry.remove_non_basic(var.clone());
        }
    }

    // TODO: improve this function, use leaving index or pivot_row instead of leaving var.
    pub fn pivot(&mut self, entering: &DictVarRef, leaving: &DictVarRef) {
        // Step 1: Find the index of the leaving variable in the entries
        let leaving_idx = self
            .entries
            .iter()
            .position(|entry| &entry.get_basic_var() == leaving);

        // Ensure the leaving variable is present in the dictionary
        if let Some(leaving_idx) = leaving_idx {
            let pivot_row = &mut self.entries[leaving_idx];
            pivot_row.switch_to_basic(entering.clone());

            // TODO: this is hot fix, consider remove this later.
            let pivot_row = pivot_row.clone();

            self.entries.iter_mut().for_each(|entry| {
                entry.replace_non_basic_with_expr(entering.clone(), &pivot_row.get_expr());
            });

            self.objective
                .replace_var_with_expr(entering.clone(), &pivot_row.get_expr());
        } else {
            panic!("Leaving variable not found in the dictionary.");
        }
    }

    fn transform_expression(
        expression: &LinearExpr<StdVarRef>,
        variable_map: &VariableMap,
    ) -> LinearExpr<DictVarRef> {
        let std_terms = expression
            .terms
            .iter()
            .map(|(var, &coefficient)| (variable_map.get(var).unwrap().clone(), coefficient))
            .collect::<HashMap<DictVarRef, f64>>();

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
