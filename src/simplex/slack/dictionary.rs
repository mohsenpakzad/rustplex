use std::{collections::HashMap, fmt, mem};

use crate::{
    core::expression::LinearExpr,
    standardization::{standard_model::StandardModel, standard_variable::StdVar},
};

use super::{dict_entry::DictEntryRef, dict_variable::DictVarRef};

#[derive(Debug, Clone)]
pub struct SlackDictionary {
    objective: LinearExpr<DictVarRef>,
    entries: Vec<DictEntryRef>,
    non_basic_entries: HashMap<DictVarRef, Vec<DictEntryRef>>,
    variable_map: HashMap<StdVar, DictVarRef>,
}

impl SlackDictionary {
    pub fn from_standard_model(standard_model: &StandardModel) -> Self {
        let variable_map = standard_model
            .get_variables()
            .iter()
            .map(|var| (var.clone(), DictVarRef::new_non_slack(var.clone())))
            .collect::<HashMap<_, _>>();

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
            .collect::<Vec<_>>();

        let non_basic_entries = entries
            .iter()
            .flat_map(|entry| {
                entry
                    .get_non_basics()
                    .into_iter()
                    .map(move |non_basic_var| (non_basic_var, entry.clone()))
            })
            .fold(HashMap::new(), |mut acc, (var, entry)| {
                acc.entry(var).or_insert_with(Vec::new).push(entry);
                acc
            });

        let objective = standard_model
            .get_objective()
            .as_ref()
            .map(|obj| Self::transform_expression(obj.get_expr(), &variable_map))
            .unwrap();

        Self {
            objective,
            entries,
            non_basic_entries,
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

    pub fn get_variable_map(&self) -> &HashMap<StdVar, DictVarRef> {
        &self.variable_map
    }

    pub fn get_objective_value(&self) -> f64 {
        self.objective.constant
    }

    pub fn get_basic_values(&self) -> HashMap<DictVarRef, f64> {
        self.entries
            .iter()
            .map(|entry| (entry.get_basic_var().clone(), entry.get_value()))
            .collect()
    }

    pub fn get_std_values(&self) -> HashMap<StdVar, f64> {
        let basic_to_entry = self
            .entries
            .iter()
            .map(|entry| (entry.get_basic_var(), entry.clone()))
            .collect::<HashMap<_, _>>();

        self.variable_map
            .iter()
            .map(|(std_var, dict_var)| {
                (
                    std_var.clone(),
                    basic_to_entry
                        .get(dict_var)
                        .map(DictEntryRef::get_value)
                        .unwrap_or(0.0),
                )
            })
            .collect()
    }

    pub fn add_var_to_all_entries(&mut self, var: DictVarRef, coefficient: f64) {
        for entry in self.entries.iter_mut() {
            entry.add_non_basic(var.clone(), coefficient);
            self.non_basic_entries
                .entry(var.clone())
                .or_insert_with(Vec::new)
                .push(entry.clone());
        }
    }

    pub fn remove_var_from_all_entries(&mut self, var: DictVarRef) {
        for entry in self.entries.iter_mut() {
            entry.remove_non_basic(var.clone());
            self.non_basic_entries.remove(&var);
        }
    }

    pub fn pivot(&mut self, entering: &DictVarRef, leaving: &DictEntryRef) {
        leaving.switch_to_basic(entering.clone());
        let leaving_expr = leaving.get_expr();

        // Update entries that contain entering variable
        self.non_basic_entries
            .get(&entering)
            .unwrap()
            .iter()
            .for_each(|entry_contains_entering| {
                entry_contains_entering
                    .replace_non_basic_with_expr(entering.clone(), &leaving_expr);
            });

        // Update objective
        self.objective
            .replace_var_with_expr(entering.clone(), &leaving_expr);
    }

    fn transform_expression(
        expression: &LinearExpr<StdVar>,
        variable_map: &HashMap<StdVar, DictVarRef>,
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
