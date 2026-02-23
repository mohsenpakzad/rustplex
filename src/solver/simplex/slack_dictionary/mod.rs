pub mod row;
pub mod variable;

use std::{fmt, mem};
use slotmap::{DenseSlotMap, SecondaryMap};

use crate::{
    modeling::expression::LinearExpr, 
    standard::{model::StandardModel, variable::StandardVariableKey},
    solver::simplex::slack_dictionary::{
        row::{DictionaryRow, DictionaryRowKey},
        variable::{DictionaryVariableKey, DictionaryVariable}
    }, 
};

#[derive(Debug, Clone)]
pub struct SlackDictionary {
    variables: DenseSlotMap<DictionaryVariableKey, DictionaryVariable>,
    objective: LinearExpr<DictionaryVariableKey>,
    entries: DenseSlotMap<DictionaryRowKey, DictionaryRow>,
    mapping: SecondaryMap<StandardVariableKey, DictionaryVariableKey>,
}

impl SlackDictionary {
    pub fn from_standard_model(standard_model: &StandardModel) -> Self {
        let mut variables = DenseSlotMap::with_key();
        let mut mapping = SecondaryMap::new();

        for var_key in standard_model.variables().keys() {
            let dict_key = variables.insert(DictionaryVariable::new_non_slack(var_key));
            mapping.insert(var_key, dict_key);
        }

        let mut entries = DenseSlotMap::with_key();
        for (index, constraint) in standard_model.constraints().values().enumerate() {
            let dict_key = variables.insert(DictionaryVariable::new_slack(index));
            entries.insert(DictionaryRow::new(
                dict_key,
                Self::transform_expression(
                    &(constraint.rhs() - constraint.lhs()),
                    &mapping
                )
            ));
        }

        let objective = standard_model
            .objective()
            .as_ref()
            .map(|obj| Self::transform_expression(obj.expr(), &mapping))
            .unwrap();

        Self {
            variables,
            objective,
            entries,
            mapping,
        }
    }

    pub fn set_objective(&mut self, objective: LinearExpr<DictionaryVariableKey>) {
        self.objective = objective;
    }

    pub fn replace_objective(&mut self, new_objective: LinearExpr<DictionaryVariableKey>) -> LinearExpr<DictionaryVariableKey> {
        mem::replace(&mut self.objective, new_objective)
    }

    pub fn variables(&self) -> &DenseSlotMap<DictionaryVariableKey, DictionaryVariable> {
        &self.variables
    }

    pub fn variables_mut(&mut self) -> &mut DenseSlotMap<DictionaryVariableKey, DictionaryVariable> {
        &mut self.variables
    }

    pub fn objective(&self) -> &LinearExpr<DictionaryVariableKey> {
        &self.objective
    }

    pub fn entries(&self) -> &DenseSlotMap<DictionaryRowKey, DictionaryRow> {
        &self.entries
    }

    pub fn mapping(&self) -> &SecondaryMap<StandardVariableKey, DictionaryVariableKey> {
        &self.mapping
    }

    pub fn objective_value(&self) -> f64 {
        self.objective.constant
    }

    pub fn basic_values(&self) -> SecondaryMap<DictionaryVariableKey, f64> {
        self.entries
            .values()
            .map(|entry| (entry.basic_var().clone(), entry.value()))
            .collect()
    }

    pub fn std_values(&self) -> SecondaryMap<StandardVariableKey, f64> {
        let basic_to_entry = self
            .entries
            .values()
            .map(|entry| (entry.basic_var(), entry.clone()))
            .collect::<SecondaryMap<_, _>>();

        self.mapping
            .iter()
            .map(|(std_var, dict_var)| {
                (
                    std_var,
                    basic_to_entry
                        .get(*dict_var)
                        .map(DictionaryRow::value)
                        .unwrap_or(0.0),
                )
            })
            .collect()
    }

    pub fn add_var_to_all_entries(&mut self, var: DictionaryVariableKey, coefficient: f64) {
        for entry in self.entries.values_mut() {
            entry.add_non_basic(var.clone(), coefficient);
        }
    }

    pub fn remove_var_from_all_entries(&mut self, var: DictionaryVariableKey) {
        for entry in self.entries.values_mut() {
            entry.remove_non_basic(var.clone());
        }
    }

    pub fn remove_entry(&mut self, key: DictionaryRowKey) {
        self.entries.remove(key);
    }

    pub fn pivot(&mut self, entering: DictionaryVariableKey, leaving_key: DictionaryRowKey) {
        // Get a mutable reference to the leaving entry in the arena and update its basis
        let leaving_entry = self.entries.get_mut(leaving_key).unwrap();
        leaving_entry.switch_to_basic(entering);
        
        // Clone the properties we need to avoid borrow-checker conflicts in the next loop
        let leaving_expr = leaving_entry.expr();
        let new_basic_var = leaving_entry.basic_var();

        // Iterate over ALL entries mutably to substitute the expression
        for entry in self.entries.values_mut() {
            // We compare basic variables to identify if it's the same row.
            if entry.basic_var() != new_basic_var {
                entry.replace_non_basic_with_expr(entering, &leaving_expr);
            }
        }

        // Update the objective
        self.objective.replace_var_with_expr(entering, &leaving_expr);
    }

    fn transform_expression(
        expression: &LinearExpr<StandardVariableKey>,
        variable_map: &SecondaryMap<StandardVariableKey, DictionaryVariableKey>,
    ) -> LinearExpr<DictionaryVariableKey> {
        let std_terms = expression
            .terms
            .iter()
            .map(|(var, coefficient)| (variable_map.get(*var).unwrap().clone(), *coefficient))
            .collect::<Vec<(DictionaryVariableKey, f64)>>();

        LinearExpr::with_terms_and_constant(std_terms, expression.constant)
    }
}

impl fmt::Display for SlackDictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the objective
        writeln!(f, "Objective = {}", self.objective)?;

        // Display the entires
        for entry in self.entries.values() {
            writeln!(f, "{}", entry)?;
        }
        Ok(())
    }
}
