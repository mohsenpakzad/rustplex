use std::{fmt, mem};
use slotmap::new_key_type;

use crate::{
    common::expression::LinearExpr,
    solver::simplex::slack_dictionary::variable::DictionaryVariableKey
};

new_key_type! {
    pub struct DictionaryRowKey;
}

impl fmt::Display for DictionaryRowKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DictionaryRowKey({:?})", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct DictionaryRow {
    basic_var: DictionaryVariableKey,
    non_basics_expr: LinearExpr<DictionaryVariableKey>,
}

impl DictionaryRow {
    /// Creates a new reference to a dictionary entry.
    pub fn new(basic_var: DictionaryVariableKey, non_basics_expr: LinearExpr<DictionaryVariableKey>) -> Self {
        DictionaryRow {
            basic_var,
            non_basics_expr,
        }
    }

    /// Adds a non-basic variable with a given coefficient to the expression.
    pub fn add_non_basic(&mut self, var: DictionaryVariableKey, coefficient: f64) {
        self.non_basics_expr.add_term(var, coefficient);
    }

    /// Removes a non-basic variable from the expression and
    /// returns its coefficient if it existed.
    pub fn remove_non_basic(&mut self, var: DictionaryVariableKey) -> Option<f64> {
        self.non_basics_expr.remove_term(&var)
    }

    /// Retrieves the coefficient of a non-basic variable from the non-basic expression.
    pub fn non_basic_coefficient(&self, var: &DictionaryVariableKey) -> f64 {
        self.non_basics_expr.coefficient(var)
    }

    /// Replaces a non-basic variable with an expression,
    /// scaling the new expression by the old variable's coefficient.
    pub fn replace_non_basic_with_expr(
        &mut self,
        var: DictionaryVariableKey,
        replacement_expr: &LinearExpr<DictionaryVariableKey>,
    ) -> Option<f64> {
        self.non_basics_expr.replace_var_with_expr(var, replacement_expr)
    }

    /// Switches the given non-basic variable to a basic variable,
    /// scaling the expression and setting the old basic variable as non-basic.
    pub fn switch_to_basic(&mut self, non_basic_var: DictionaryVariableKey) -> Option<f64> {
        if let Some(coefficient) = self.non_basics_expr.remove_term(&non_basic_var) {
            let old_basic_var = mem::replace(&mut self.basic_var, non_basic_var);

            self.non_basics_expr.add_term(old_basic_var, -1.0);
            self.non_basics_expr.scale(1.0 / -coefficient);
            Some(coefficient)
        } else {
            None
        }
    }

    /// Gets the basic variable of the dictionary entry.
    pub fn basic_var(&self) -> DictionaryVariableKey {
        self.basic_var
    }

    /// Gets the value (constant) of the dictionary entry.
    pub fn value(&self) -> f64 {
        self.non_basics_expr.constant
    }

    /// Gets the expression of non-basic variables in the dictionary entry.
    pub fn expr(&self) -> LinearExpr<DictionaryVariableKey> {
        self.non_basics_expr.clone()
    }
}

impl fmt::Display for DictionaryRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} = {}",
            self.basic_var,
            self.non_basics_expr
        )
    }
}
